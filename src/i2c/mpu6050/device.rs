use rsiot::components_config::i2c_master::I2cAddress;
use rsiot::components_config::master_device::{FieldbusDiagMsg, ResponseResult};
use rsiot::executor::MsgBusInput;
use tracing::{debug, info, trace, warn};

use super::buffer::{CalibrationOffsets, CalibrationProcess};
use super::{
    async_trait, buffer::WriteData, mpsc, AfsSel, BitField, BitView, Buffer, ConfigPeriodicRequest,
    DeviceBase, DeviceTrait, Duration, FieldbusRequest, FieldbusResponse, FsSel, MPU6050Registers,
    Message, Msb0, MsgDataBound, Operation, RequestKind, Result,
};

/// Датчик температуры и влажности AHT10
#[derive(Clone, Debug)]
pub struct Device<TMsg> {
    /// Адрес зависит от AD0:
    /// - GND - 0x68
    /// - VCC - 0x69
    pub address: I2cAddress,

    pub request_period: Duration,

    /// Преобразование данных из буфера в исходящие сообщения
    pub fn_output: fn(&mut Buffer) -> Vec<TMsg>,

    pub gyro_full_range: FsSel,
    pub accel_full_range: AfsSel,

    pub default_calibration_offset_accel_x: i16,
    pub default_calibration_offset_accel_y: i16,
    pub default_calibration_offset_accel_z: i16,
    pub default_calibration_offset_gyro_x: i16,
    pub default_calibration_offset_gyro_y: i16,
    pub default_calibration_offset_gyro_z: i16,
    pub default_calibration_start: bool,
}

#[async_trait]
impl<TMsg> DeviceTrait<TMsg, FieldbusRequest, FieldbusResponse> for Device<TMsg>
where
    TMsg: MsgDataBound + 'static,
{
    async fn spawn(
        self: Box<Self>,
        ch_rx_msgbus_to_device: MsgBusInput<TMsg>,
        ch_tx_device_to_fieldbus: mpsc::Sender<FieldbusRequest>,
        ch_rx_fieldbus_to_device: mpsc::Receiver<FieldbusResponse>,
        ch_tx_device_to_msgbus: mpsc::Sender<Message<TMsg>>,
        ch_tx_device_to_diag: mpsc::Sender<FieldbusDiagMsg>,
    ) -> Result<()> {
        let device = DeviceBase {
            fn_init_requests: |buffer: &Buffer| {
                let mut requests = vec![];

                // Читаем калибровочные смещения
                let req = FieldbusRequest::new(
                    buffer.address,
                    RequestKind::ReadCalibrationOffsets,
                    vec![MPU6050Registers::read_calibration_offsets()],
                );
                requests.push(req);

                // Начальная настройка
                let mut operations = vec![];

                let op = Operation::Write {
                    write_data: vec![0x6B, 0x00],
                };
                operations.push(op);

                let op = MPU6050Registers::write_gyro_config(&buffer.write_data.gyro_full_range);
                operations.push(op);

                let op = MPU6050Registers::write_accel_config(&buffer.write_data.accel_full_range);
                operations.push(op);

                let ops = MPU6050Registers::write_calibration_offsets(
                    buffer.write_data.calibration_offsets.accel_x,
                    buffer.write_data.calibration_offsets.accel_y,
                    buffer.write_data.calibration_offsets.accel_z,
                    buffer.write_data.calibration_offsets.gyro_x,
                    buffer.write_data.calibration_offsets.gyro_y,
                    buffer.write_data.calibration_offsets.gyro_z,
                );
                operations.extend(ops);

                let req = FieldbusRequest::new(buffer.address, RequestKind::Init, operations);
                requests.push(req);

                // Читаем записанные настройки
                let req = FieldbusRequest::new(
                    buffer.address,
                    RequestKind::ReadFullScaleConfig,
                    vec![MPU6050Registers::read_config()],
                );
                requests.push(req);

                requests
            },
            periodic_requests: vec![
                ConfigPeriodicRequest {
                    period: self.request_period,
                    fn_requests: |buffer: &Buffer| {
                        let mut requests = vec![];

                        let req = FieldbusRequest::new(
                            buffer.address,
                            RequestKind::ReadValues,
                            vec![
                                Operation::WriteRead {
                                    write_data: vec![0x3B],
                                    read_size: 14,
                                }, // Operation::Write {
                                   //     write_data: vec![0x3B],
                                   // },
                                   // Operation::Read { read_size: 14 },
                            ],
                        );
                        requests.push(req);

                        Ok(requests)
                    },
                },
                ConfigPeriodicRequest {
                    period: Duration::from_millis(100),
                    fn_requests: |_buffer: &Buffer| Ok(vec![]),
                },
            ],
            fn_msgs_to_buffer: |_msg, _buffer| (),
            buffer_to_request_period: Duration::from_millis(100),
            fn_buffer_to_request: |buffer: &Buffer| {
                let mut requests = vec![];

                // Записываем калибровочные коэффициенты, если они изменились
                if buffer.read_data.calibration_offsets != buffer.write_data.calibration_offsets {
                    info!(
                        "Calibration offsets changed: {:?}; {:?}",
                        buffer.write_data.calibration_offsets, buffer.read_data.calibration_offsets
                    );
                    let req = FieldbusRequest::new(
                        buffer.address,
                        RequestKind::WriteCalibrationOffsets,
                        MPU6050Registers::write_calibration_offsets(
                            buffer.write_data.calibration_offsets.accel_x,
                            buffer.write_data.calibration_offsets.accel_y,
                            buffer.write_data.calibration_offsets.accel_z,
                            buffer.write_data.calibration_offsets.gyro_x,
                            buffer.write_data.calibration_offsets.gyro_y,
                            buffer.write_data.calibration_offsets.gyro_z,
                        ),
                    );
                    requests.push(req);

                    let req = FieldbusRequest::new(
                        buffer.address,
                        RequestKind::ReadCalibrationOffsets,
                        vec![MPU6050Registers::read_calibration_offsets()],
                    );
                    requests.push(req);
                }

                if (buffer.read_data.gyro_full_range != buffer.write_data.gyro_full_range)
                    || (buffer.read_data.accel_full_range != buffer.write_data.accel_full_range)
                {
                    debug!(
                        "Updating full scale configuration; {:?}; {:?}; {:?}; {:?}",
                        buffer.write_data.gyro_full_range,
                        buffer.read_data.gyro_full_range,
                        buffer.write_data.accel_full_range,
                        buffer.read_data.accel_full_range
                    );
                    let req = FieldbusRequest::new(
                        buffer.address,
                        RequestKind::WriteFullScaleConfig,
                        vec![
                            MPU6050Registers::write_gyro_config(&buffer.write_data.gyro_full_range),
                            MPU6050Registers::write_accel_config(
                                &buffer.write_data.accel_full_range,
                            ),
                        ],
                    );
                    requests.push(req);

                    let req = FieldbusRequest::new(
                        buffer.address,
                        RequestKind::ReadFullScaleConfig,
                        vec![MPU6050Registers::read_config()],
                    );
                    requests.push(req);
                }

                Ok(requests)
            },
            fn_response_to_buffer: |response: FieldbusResponse, buffer: &mut Buffer| {
                trace!("Response: {:?}", response);

                let request_kind: RequestKind = response.request_kind.try_into()?;

                let payload = match response.payload {
                    Ok(payload) => payload,
                    Err(err) => {
                        warn!("Error reading MPU-6050: {}", err);
                        return ResponseResult::error(err);
                    }
                };

                match request_kind {
                    RequestKind::Init => ResponseResult::ok_init_completed(),

                    RequestKind::ReadFullScaleConfig => {
                        debug!("Response: read full scale configuration from MPU-6050");

                        let bits = payload[0].view_bits::<Msb0>();

                        buffer.read_data.gyro_full_range = match (bits[3], bits[4]) {
                            (false, false) => FsSel::_250DPS,
                            (false, true) => FsSel::_500DPS,
                            (true, false) => FsSel::_1000DPS,
                            (true, true) => FsSel::_2000DPS,
                        };
                        buffer.read_data.accel_full_range = match (bits[11], bits[12]) {
                            (false, false) => AfsSel::_2G,
                            (false, true) => AfsSel::_4G,
                            (true, false) => AfsSel::_8G,
                            (true, true) => AfsSel::_16G,
                        };

                        ResponseResult::ok()
                    }

                    RequestKind::WriteFullScaleConfig => {
                        debug!("Response: write full scale configuration to MPU-6050");
                        ResponseResult::ok()
                    }

                    RequestKind::ReadValues => {
                        let accel_full_range = &buffer.write_data.accel_full_range;
                        let gyro_full_range = &buffer.write_data.gyro_full_range;

                        let bits = payload[0].view_bits::<Msb0>();

                        let accel_x_raw = bits[0..16].load_be::<i16>();
                        buffer.read_data.accel_x_raw = accel_x_raw;
                        buffer.read_data.accel_x = accel_from_raw(accel_x_raw, accel_full_range);

                        let accel_y_raw = bits[16..32].load_be::<i16>();
                        buffer.read_data.accel_y_raw = accel_y_raw;
                        buffer.read_data.accel_y = accel_from_raw(accel_y_raw, accel_full_range);

                        let accel_z_raw = bits[32..48].load_be::<i16>();
                        buffer.read_data.accel_z_raw = accel_z_raw;
                        buffer.read_data.accel_z = accel_from_raw(accel_z_raw, accel_full_range);

                        let temp = bits[48..64].load_be::<i16>();
                        buffer.read_data.temperature = temp_from_raw(temp);

                        let gyro_x_raw = bits[64..80].load_be::<i16>();
                        buffer.read_data.gyro_x_raw = gyro_x_raw;
                        buffer.read_data.gyro_x = gyro_from_raw(gyro_x_raw, gyro_full_range);

                        let gyro_y_raw = bits[80..96].load_be::<i16>();
                        buffer.read_data.gyro_y_raw = gyro_y_raw;
                        buffer.read_data.gyro_y = gyro_from_raw(gyro_y_raw, gyro_full_range);

                        let gyro_z_raw = bits[96..112].load_be::<i16>();
                        buffer.read_data.gyro_z_raw = gyro_z_raw;
                        buffer.read_data.gyro_z = gyro_from_raw(gyro_z_raw, gyro_full_range);

                        let res = calibration_process(buffer);
                        match res {
                            true => ResponseResult::ok_need_request(),
                            false => ResponseResult::ok(),
                        }
                    }

                    RequestKind::ReadCalibrationOffsets => {
                        let bits = payload[0].view_bits::<Msb0>();

                        let offset_accel_x = bits[0..16].load_be::<i16>();
                        buffer.read_data.calibration_offsets.accel_x = offset_accel_x;

                        let offset_accel_y = bits[16..32].load_be::<i16>();
                        buffer.read_data.calibration_offsets.accel_y = offset_accel_y;

                        let offset_accel_z = bits[32..48].load_be::<i16>();
                        buffer.read_data.calibration_offsets.accel_z = offset_accel_z;

                        let offset_gyro_x = bits[104..120].load_be::<i16>();
                        buffer.read_data.calibration_offsets.gyro_x = offset_gyro_x;

                        let offset_gyro_y = bits[120..136].load_be::<i16>();
                        buffer.read_data.calibration_offsets.gyro_y = offset_gyro_y;

                        let offset_gyro_z = bits[136..152].load_be::<i16>();
                        buffer.read_data.calibration_offsets.gyro_z = offset_gyro_z;

                        info!(
                            r#"
Current calibration data:
Accel_X: {offset_accel_x}
Accel_Y: {offset_accel_y}
Accel_Z: {offset_accel_z}
Gyro_X: {offset_gyro_x}
Gyro_Y: {offset_gyro_y}
Gyro_Z: {offset_gyro_z}
"#
                        );
                        ResponseResult::ok()
                    }

                    RequestKind::WriteCalibrationOffsets => {
                        debug!("Response: write new calibration offsets");
                        ResponseResult::ok()
                    }
                }
            },
            fn_buffer_to_msgs: self.fn_output,
            buffer_default: Buffer {
                address: self.address,
                write_data: WriteData {
                    gyro_full_range: self.gyro_full_range,
                    accel_full_range: self.accel_full_range,
                    calibration_offsets: CalibrationOffsets {
                        accel_x: self.default_calibration_offset_accel_x,
                        accel_y: self.default_calibration_offset_accel_y,
                        accel_z: self.default_calibration_offset_accel_z,
                        gyro_x: self.default_calibration_offset_gyro_x,
                        gyro_y: self.default_calibration_offset_gyro_y,
                        gyro_z: self.default_calibration_offset_gyro_z,
                    },
                },
                calibration_process: CalibrationProcess {
                    start: self.default_calibration_start,
                    ..Default::default()
                },
                ..Default::default()
            },
        };
        device
            .spawn(
                "MPU6050",
                ch_rx_msgbus_to_device,
                ch_tx_device_to_fieldbus,
                ch_rx_fieldbus_to_device,
                ch_tx_device_to_msgbus,
                ch_tx_device_to_diag,
            )
            .await?;
        Ok(())
    }
}

fn accel_from_raw(raw_value: i16, afs_sel: &AfsSel) -> f64 {
    match afs_sel {
        AfsSel::_2G => raw_value as f64 / 16384.0,
        AfsSel::_4G => raw_value as f64 / 8192.0,
        AfsSel::_8G => raw_value as f64 / 4096.0,
        AfsSel::_16G => raw_value as f64 / 2048.0,
    }
}

fn temp_from_raw(raw_value: i16) -> f64 {
    raw_value as f64 / 340.0 + 36.53
}

fn gyro_from_raw(raw_value: i16, gfs_sel: &FsSel) -> f64 {
    match gfs_sel {
        FsSel::_250DPS => raw_value as f64 / 131.0,
        FsSel::_500DPS => raw_value as f64 / 65.5,
        FsSel::_1000DPS => raw_value as f64 / 32.8,
        FsSel::_2000DPS => raw_value as f64 / 16.4,
    }
}

fn calibration_process(buffer: &mut Buffer) -> bool {
    let cp = &mut buffer.calibration_process;

    match (cp.start, cp.run) {
        // Выходим из функции
        (false, false) => return false,

        // Пришла команда на запуск
        (true, false) => {
            info!("Starting calibration process");
            cp.run = true;

            // Сбрасываем калибровочные смещения
            buffer.write_data.calibration_offsets = Default::default();

            // Сохраняем текущие настройки полного диапазона
            cp.gyro_full_range = buffer.write_data.gyro_full_range;
            buffer.write_data.gyro_full_range = Default::default();
            cp.accel_full_range = buffer.write_data.accel_full_range;
            buffer.write_data.accel_full_range = Default::default();

            cp.buffer_accel_x = 0;
            cp.buffer_accel_y = 0;
            cp.buffer_accel_z = 0;
            cp.buffer_gyro_x = 0;
            cp.buffer_gyro_y = 0;
            cp.buffer_gyro_z = 0;
            cp.current_measurement_in_step = 0;
            cp.current_step = 0;
            return true;
        }

        // Идёт процесс калибровки
        (false, true) => (),

        // Идёт процесс калибровки
        (true, true) => {
            cp.start = false;
        }
    };

    cp.current_measurement_in_step += 1;
    if cp.current_measurement_in_step <= cp.skip_measurements {
        return false;
    }

    cp.buffer_accel_x += buffer.read_data.accel_x_raw as i64;
    cp.buffer_accel_y += buffer.read_data.accel_y_raw as i64;
    cp.buffer_accel_z += buffer.read_data.accel_z_raw as i64;
    cp.buffer_gyro_x += buffer.read_data.gyro_x_raw as i64;
    cp.buffer_gyro_y += buffer.read_data.gyro_y_raw as i64;
    cp.buffer_gyro_z += buffer.read_data.gyro_z_raw as i64;

    if cp.current_measurement_in_step < cp.skip_measurements + cp.buffer_count {
        return false;
    }

    // Окончание шага калибровки
    let accel_x_new =
        cp.prev_offset_accel_x - (cp.buffer_accel_x as f64 / cp.buffer_count as f64) as i64;
    buffer.write_data.calibration_offsets.accel_x = (accel_x_new / 8) as i16;
    cp.prev_offset_accel_x = accel_x_new;

    let accel_y_new =
        cp.prev_offset_accel_y - (cp.buffer_accel_y as f64 / cp.buffer_count as f64) as i64;
    buffer.write_data.calibration_offsets.accel_y = (accel_y_new / 8) as i16;
    cp.prev_offset_accel_y = accel_y_new;

    let accel_z_new =
        cp.prev_offset_accel_z - (cp.buffer_accel_z as f64 / cp.buffer_count as f64) as i64;
    let accel_z_new = accel_z_new + 16384;
    buffer.write_data.calibration_offsets.accel_z = (accel_z_new / 8) as i16;
    cp.prev_offset_accel_z = accel_z_new;

    let gyro_x_new =
        cp.prev_offset_gyro_x - (cp.buffer_gyro_x as f64 / cp.buffer_count as f64) as i64;
    buffer.write_data.calibration_offsets.gyro_x = (gyro_x_new / 4) as i16;
    cp.prev_offset_gyro_x = gyro_x_new;

    let gyro_y_new =
        cp.prev_offset_gyro_y - (cp.buffer_gyro_y as f64 / cp.buffer_count as f64) as i64;
    buffer.write_data.calibration_offsets.gyro_y = (gyro_y_new / 4) as i16;
    cp.prev_offset_gyro_y = gyro_y_new;

    let gyro_z_new =
        cp.prev_offset_gyro_z - (cp.buffer_gyro_z as f64 / cp.buffer_count as f64) as i64;
    buffer.write_data.calibration_offsets.gyro_z = (gyro_z_new / 4) as i16;
    cp.prev_offset_gyro_z = gyro_z_new;

    cp.current_step += 1;

    if cp.current_step < cp.steps {
        info!("Calibration: step complete");
        cp.current_measurement_in_step = 0;
        cp.buffer_accel_x = 0;
        cp.buffer_accel_y = 0;
        cp.buffer_accel_z = 0;
        cp.buffer_gyro_x = 0;
        cp.buffer_gyro_y = 0;
        cp.buffer_gyro_z = 0;
        return false;
    }

    info!("Calibration: complete");
    cp.run = false;
    buffer.write_data.gyro_full_range = cp.gyro_full_range;
    buffer.write_data.accel_full_range = cp.accel_full_range;

    false
}
