use std::f32::consts::PI;

use rsiot::components_config::i2c_master::I2cAddress;
use rsiot::components_config::master_device::{FieldbusDiagMsg, ResponseResult};
use rsiot::executor::MsgBusInput;
use tracing::{debug, info, trace, warn};

use crate::i2c::device_id;

use super::{
    async_trait,
    buffer::{CalibrationOffsets, CalibrationProcess, Config, WriteData},
    mpsc, physics,
    registers::{self, Operations},
    AccelFullScale, BitField, BitView, Buffer, ConfigPeriodicRequest, DeviceBase, DeviceTrait,
    Duration, FieldbusRequest, FieldbusResponse, GyroFullScale, Message, Msb0, MsgDataBound,
    Operation, RequestKind, Result, DEVICE_NAME,
};

const ACCEL_DISCREPANCY: f32 = 0.2;
const GYRO_DISCREPANCY: f32 = 100.0;

/// Датчик температуры и влажности AHT10
#[derive(Clone, Debug)]
pub struct Device<TMsg> {
    /// Адрес зависит от AD0:
    /// - GND - 0x68
    /// - VCC - 0x69
    pub address: I2cAddress,

    /// Период опроса датчика
    pub request_period: Duration,

    /// Преобразование данных из буфера в исходящие сообщения
    pub fn_output: fn(&mut Buffer) -> Vec<TMsg>,

    /// Диапазон измерения угловой скорости
    pub gyro_full_range: GyroFullScale,

    /// Диапазон измерения ускорения
    pub accel_full_range: AccelFullScale,

    /// Значение калибровки ускорения оси X
    pub calibration_accel_x: i16,

    /// Значение калибровки ускорения оси Y
    pub calibration_accel_y: i16,

    /// Значение калибровки ускорения оси Z
    pub calibration_accel_z: i16,

    /// Значение калибровки угловой скорости оси X
    pub calibration_gyro_x: i16,

    /// Значение калибровки угловой скорости оси Y
    pub calibration_gyro_y: i16,

    /// Значение калибровки угловой скорости оси Z
    pub calibration_gyro_z: i16,

    /// true - выполняется калибровка датчика
    pub start_calibration: bool,

    /// Определение положения в пространстве с помощью DMP
    pub dmp_enabled: bool,
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
        let device_id = device_id(DEVICE_NAME, self.address);

        let device = DeviceBase {
            fn_init_requests,
            periodic_requests: periodic_requests(
                self.request_period,
                self.dmp_enabled,
                self.start_calibration,
            ),
            fn_msgs_to_buffer: |_msg, _buffer| (),
            buffer_to_request_period: Duration::from_millis(100),
            fn_buffer_to_request,
            fn_response_to_buffer,
            fn_buffer_to_msgs: self.fn_output,
            buffer_default: Buffer {
                config: Config {
                    address: self.address,
                    dmp_enabled: self.dmp_enabled,
                },
                write_data: WriteData {
                    gyro_full_range: self.gyro_full_range,
                    accel_full_range: self.accel_full_range,
                    calibration_offsets: CalibrationOffsets {
                        accel_x: self.calibration_accel_x,
                        accel_y: self.calibration_accel_y,
                        accel_z: self.calibration_accel_z,
                        gyro_x: self.calibration_gyro_x,
                        gyro_y: self.calibration_gyro_y,
                        gyro_z: self.calibration_gyro_z,
                    },
                },
                calibration_process: CalibrationProcess {
                    start: self.start_calibration,
                    ..Default::default()
                },
                ..Default::default()
            },
        };
        device
            .spawn(
                device_id,
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

fn fn_init_requests(buffer: &Buffer) -> Vec<FieldbusRequest> {
    let mut requests = vec![];

    // Читаем калибровочные смещения
    let req = FieldbusRequest::new(
        buffer.config.address,
        RequestKind::ReadCalibrationOffsets,
        vec![Operations::read_calibration_offsets()],
    );
    requests.push(req);

    // Начальная настройка
    let mut operations = vec![];

    // Сброс устройства
    let op = Operations::write_pwr_mgmt_1(registers::pwr_mgmt_1::PwrMgmt1 {
        device_reset: true,
        sleep: false,
        cycle: false,
        temp_dis: false,
        clc_sel: registers::clock_source::ClockSource::Internal,
    });
    operations.push(op);
    operations.push(Operation::Delay {
        delay: Duration::from_millis(200),
    });

    // Конфигурация после сброса
    let op = Operations::write_pwr_mgmt_1(registers::pwr_mgmt_1::PwrMgmt1 {
        device_reset: false,
        sleep: false,
        cycle: false,
        temp_dis: false,
        clc_sel: registers::clock_source::ClockSource::Xgyro,
    });
    operations.push(op);

    // Сброс показаний
    let op = Operations::write_user_ctrl(registers::user_ctrl::UserCtrl {
        reserved_7: false,
        fifo_en: false,
        i2c_mst_en: false,
        i2c_if_dis: false,
        fifo_reset: false,
        i2c_mst_reset: false,
        sig_cond_reset: true,
    });
    operations.push(op);
    operations.push(Operation::Delay {
        delay: Duration::from_millis(200),
    });

    // Загрузка DMP
    if buffer.config.dmp_enabled {
        // Отключаем прерывания
        let op = Operations::write_int_enable(registers::int_enable::IntEnable {
            fifo_oflow_en: false,
            i2c_mst_int_en: false,
            data_rdy_en: false,
        });
        operations.push(op);

        // Отключаем FIFO
        let op = Operations::write_fifo_en(registers::fifo_en::FifoEn {
            temp_fifo_en: false,
            xg_fifo_en: false,
            yg_fifo_en: false,
            zg_fifo_en: false,
            accel_fifo_en: false,
            slv2_fifo_en: false,
            slv1_fifo_en: false,
            slv0_fifo_en: false,
        });
        operations.push(op);

        // Диапазон ускорений
        let op = Operations::write_accel_config(registers::accel_config::AccelConfig {
            xa_st: false,
            ya_st: false,
            za_st: false,
            full_scale: AccelFullScale::G2,
        });
        operations.push(op);

        // Делитель частоты
        let op = Operations::write_sample_rate_divider(4);
        operations.push(op);

        // Конфигурация
        let op = Operations::write_config(registers::config::Config {
            digital_low_pass_filter:
                registers::digital_low_pass_filter::DigitalLowPassFilter::Filter6,
        });
        operations.push(op);

        // Загрузка DMP
        let dmp_ops = Operations::write_dmp_firmware();
        operations.extend(dmp_ops);

        // Активация DMP
        let boot_ops = Operations::write_boot_firmware();
        operations.push(boot_ops);

        // Диапазон угловых скоростей
        let op = Operations::write_gyro_config(registers::gyro_config::GyroConfig {
            xg_st: false,
            yg_st: false,
            zg_st: false,
            full_scale: GyroFullScale::Deg2000,
        });
        operations.push(op);

        // Включить FIFO и сбросить FIFO
        let op = Operations::write_user_ctrl(registers::user_ctrl::UserCtrl {
            reserved_7: false,
            fifo_en: true,
            i2c_mst_en: false,
            i2c_if_dis: false,
            fifo_reset: true,
            i2c_mst_reset: false,
            sig_cond_reset: false,
        });
        operations.push(op);

        // Включить DMP
        let op = Operations::write_user_ctrl(registers::user_ctrl::UserCtrl {
            reserved_7: true,
            fifo_en: true,
            i2c_mst_en: false,
            i2c_if_dis: false,
            fifo_reset: false,
            i2c_mst_reset: false,
            sig_cond_reset: false,
        });
        operations.push(op);
    }

    // let op = Operations::write_gyro_config(registers::gyro_config::GyroConfig {
    //     xg_st: false,
    //     yg_st: false,
    //     zg_st: false,
    //     full_scale: buffer.write_data.gyro_full_range,
    // });
    // operations.push(op);

    // let op = Operations::write_accel_config(registers::accel_config::AccelConfig {
    //     xa_st: false,
    //     ya_st: false,
    //     za_st: false,
    //     full_scale: buffer.write_data.accel_full_range,
    // });
    // operations.push(op);

    let ops = Operations::write_calibration_offsets(
        buffer.write_data.calibration_offsets.accel_x,
        buffer.write_data.calibration_offsets.accel_y,
        buffer.write_data.calibration_offsets.accel_z,
        buffer.write_data.calibration_offsets.gyro_x,
        buffer.write_data.calibration_offsets.gyro_y,
        buffer.write_data.calibration_offsets.gyro_z,
    );
    operations.extend(ops);

    let req = FieldbusRequest::new(buffer.config.address, RequestKind::Init, operations);
    requests.push(req);

    // Читаем записанные настройки
    let req = FieldbusRequest::new(
        buffer.config.address,
        RequestKind::ReadFullScaleConfig,
        vec![Operations::read_config()],
    );
    requests.push(req);

    requests
}

fn periodic_requests(
    request_period: Duration,
    dmp_enabled: bool,
    calibration_start: bool,
) -> Vec<ConfigPeriodicRequest<FieldbusRequest, Buffer>> {
    let req = match dmp_enabled && !calibration_start {
        true => ConfigPeriodicRequest {
            period: request_period,
            fn_requests: |buffer: &Buffer| {
                let mut requests = vec![];

                let req = FieldbusRequest::new(
                    buffer.config.address,
                    RequestKind::ReadFifoCount,
                    vec![Operations::read_fifo_count()],
                );
                requests.push(req);

                Ok(requests)
            },
        },
        false => ConfigPeriodicRequest {
            period: request_period,
            fn_requests: |buffer: &Buffer| {
                let mut requests = vec![];

                let req = FieldbusRequest::new(
                    buffer.config.address,
                    RequestKind::ReadValues,
                    vec![Operations::read_accel_gyro()],
                );
                requests.push(req);

                Ok(requests)
            },
        },
    };

    vec![req]
}

fn fn_buffer_to_request(buffer: &Buffer) -> anyhow::Result<Vec<FieldbusRequest>> {
    let mut requests = vec![];

    // Записываем калибровочные коэффициенты, если они изменились
    if buffer.read_data.calibration_offsets != buffer.write_data.calibration_offsets {
        info!(
            "Calibration offsets changed: {:?}; {:?}",
            buffer.write_data.calibration_offsets, buffer.read_data.calibration_offsets
        );
        let req = FieldbusRequest::new(
            buffer.config.address,
            RequestKind::WriteCalibrationOffsets,
            Operations::write_calibration_offsets(
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
            buffer.config.address,
            RequestKind::ReadCalibrationOffsets,
            vec![Operations::read_calibration_offsets()],
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
            buffer.config.address,
            RequestKind::WriteFullScaleConfig,
            vec![
                Operations::write_gyro_config(registers::gyro_config::GyroConfig {
                    xg_st: false,
                    yg_st: false,
                    zg_st: false,
                    full_scale: buffer.write_data.gyro_full_range,
                }),
                Operations::write_accel_config(registers::accel_config::AccelConfig {
                    xa_st: false,
                    ya_st: false,
                    za_st: false,
                    full_scale: buffer.write_data.accel_full_range,
                }),
            ],
        );
        requests.push(req);

        let req = FieldbusRequest::new(
            buffer.config.address,
            RequestKind::ReadFullScaleConfig,
            vec![Operations::read_config()],
        );
        requests.push(req);
    }

    // Читаем буфер DMP
    if buffer.read_data.fifo_count >= registers::DMP_FIFO_LEN {
        let req = FieldbusRequest::new(
            buffer.config.address,
            RequestKind::ReadDmpFifo,
            vec![
                Operations::read_fifo_rw(28),
                Operations::write_user_ctrl(registers::user_ctrl::UserCtrl {
                    reserved_7: true,
                    fifo_en: true,
                    i2c_mst_en: false,
                    i2c_if_dis: false,
                    fifo_reset: true,
                    i2c_mst_reset: false,
                    sig_cond_reset: false,
                }),
                Operations::read_accel_gyro(),
            ],
        );
        requests.push(req);

        let req = FieldbusRequest::new(
            buffer.config.address,
            RequestKind::ReadFifoCount,
            vec![Operations::read_fifo_count()],
        );
        requests.push(req);
    }

    Ok(requests)
}

fn fn_response_to_buffer(
    response: FieldbusResponse,
    buffer: &mut Buffer,
) -> anyhow::Result<ResponseResult> {
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
                (false, false) => GyroFullScale::Deg250,
                (false, true) => GyroFullScale::Deg500,
                (true, false) => GyroFullScale::Deg1000,
                (true, true) => GyroFullScale::Deg2000,
            };
            buffer.read_data.accel_full_range = match (bits[11], bits[12]) {
                (false, false) => AccelFullScale::G2,
                (false, true) => AccelFullScale::G4,
                (true, false) => AccelFullScale::G8,
                (true, true) => AccelFullScale::G16,
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

        RequestKind::ReadFifoCount => {
            // if payload[0].len() != 2 {
            //     return ResponseResult::error("Invalid FIFO count");
            // }
            // let fifo_count = [payload[0][0], payload[0][1]];
            let fifo_count = [payload[0][0], payload[0][1]];
            let fifo_count = u16::from_be_bytes(fifo_count) as usize;
            buffer.read_data.fifo_count = fifo_count;

            if fifo_count >= registers::DMP_FIFO_LEN {
                ResponseResult::ok_need_request()
            } else {
                ResponseResult::ok()
            }
        }

        RequestKind::ReadDmpFifo => {
            let p_dmp = &payload[0];
            let p_a_g = &payload[2];

            // 0 - read_fifo_rw --------------------------------------------------------------------
            if p_dmp.len() < registers::DMP_FIFO_LEN {
                return ResponseResult::error("FIFO count mismatch");
            }

            let len = p_dmp.len();
            let data = p_dmp[len - registers::DMP_FIFO_LEN..len].to_vec();

            let mut accel_buffer = [0_u8; 6];
            let mut gyro_buffer = [0_u8; 6];

            // Первые 16 байт - кватернион
            let quat = physics::Quaternion::from_bytes(&data[..16]);
            let Some(quat) = quat else {
                return ResponseResult::error("Load quaternion failed");
            };
            let quat = quat.normalize();

            // Convert quaternion to more intuitive Yaw, Pitch, Roll angles
            let ypr = physics::YawPitchRoll::from(quat);

            let yaw_deg = ypr.yaw * 180.0 / PI;
            let pitch_deg = ypr.pitch * 180.0 / PI;
            let roll_deg = ypr.roll * 180.0 / PI;
            trace!(
                "Angles [deg]: yaw={:.1}, pitch={:.1}, roll={:.1}",
                yaw_deg,
                pitch_deg,
                roll_deg
            );

            accel_buffer.clone_from_slice(&data[16..22]);
            let accel = physics::Accel::from_bytes(accel_buffer);
            let accel = accel.scaled(physics::AccelFullScale::G2);

            gyro_buffer.clone_from_slice(&data[22..28]);
            let gyro = physics::Gyro::from_bytes(gyro_buffer);
            let gyro = gyro.scaled(physics::GyroFullScale::Deg2000);

            // 2 - accel_gyro ----------------------------------------------------------------------
            accel_buffer.clone_from_slice(&p_a_g[0..6]);
            let accel_2 = physics::Accel::from_bytes(accel_buffer);
            let accel_2 = accel_2.scaled(physics::AccelFullScale::G2);

            gyro_buffer.clone_from_slice(&p_a_g[8..14]);
            let gyro_2 = physics::Gyro::from_bytes(gyro_buffer);
            let gyro_2 = gyro_2.scaled(physics::GyroFullScale::Deg2000);

            let wrong = (accel.x() - accel_2.x()).abs() > ACCEL_DISCREPANCY
                || (accel.y() - accel_2.y()).abs() > ACCEL_DISCREPANCY
                || (accel.z() - accel_2.z()).abs() > ACCEL_DISCREPANCY
                || (gyro.x() - gyro_2.x()).abs() > GYRO_DISCREPANCY
                || (gyro.y() - gyro_2.y()).abs() > GYRO_DISCREPANCY
                || (gyro.z() - gyro_2.z()).abs() > GYRO_DISCREPANCY;

            if wrong {
                trace!(
                    "\n{}/{}         {}/{}          {}/{}\n{}/{}         {}/{}          {}/{}",
                    accel.x(),
                    accel_2.x(),
                    accel.y(),
                    accel_2.y(),
                    accel.z(),
                    accel_2.z(),
                    gyro.x(),
                    gyro_2.x(),
                    gyro.y(),
                    gyro_2.y(),
                    gyro.z(),
                    gyro_2.z()
                );
                return ResponseResult::error("Wrong MPU6050 data");
            }

            buffer.read_data.yaw = yaw_deg as f64;
            buffer.read_data.pitch = pitch_deg as f64;
            buffer.read_data.roll = roll_deg as f64;

            buffer.read_data.accel_x = accel.x() as f64;
            buffer.read_data.accel_y = accel.y() as f64;
            buffer.read_data.accel_z = accel.z() as f64;

            buffer.read_data.gyro_x = gyro.x() as f64;
            buffer.read_data.gyro_y = gyro.y() as f64;
            buffer.read_data.gyro_z = gyro.z() as f64;

            // 2 - read_temperature ----------------------------------------------------------------
            // let temp = i16::from_be_bytes([p_temp[0], p_temp[1]]);
            // buffer.read_data.temperature = temp_from_raw(temp);

            ResponseResult::ok()
        }

        RequestKind::ResetFifoBuffer => ResponseResult::ok(),
    }
}

fn accel_from_raw(raw_value: i16, afs_sel: &AccelFullScale) -> f64 {
    match afs_sel {
        AccelFullScale::G2 => raw_value as f64 / 16384.0,
        AccelFullScale::G4 => raw_value as f64 / 8192.0,
        AccelFullScale::G8 => raw_value as f64 / 4096.0,
        AccelFullScale::G16 => raw_value as f64 / 2048.0,
    }
}

fn temp_from_raw(raw_value: i16) -> f64 {
    raw_value as f64 / 340.0 + 36.53
}

fn gyro_from_raw(raw_value: i16, gfs_sel: &GyroFullScale) -> f64 {
    match gfs_sel {
        GyroFullScale::Deg250 => raw_value as f64 / 131.0,
        GyroFullScale::Deg500 => raw_value as f64 / 65.5,
        GyroFullScale::Deg1000 => raw_value as f64 / 32.8,
        GyroFullScale::Deg2000 => raw_value as f64 / 16.4,
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
