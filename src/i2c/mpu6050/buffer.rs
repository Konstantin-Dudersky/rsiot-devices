use super::{AfsSel, BufferBound, FsSel};

/// Буфер данных
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Buffer {
    pub address: u8,
    pub write_data: WriteData,
    pub read_data: ReadData,
    pub calibration_process: CalibrationProcess,
}
impl BufferBound for Buffer {}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WriteData {
    pub calibration_offsets: CalibrationOffsets,
    pub gyro_full_range: FsSel,
    pub accel_full_range: AfsSel,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReadData {
    pub calibration_offsets: CalibrationOffsets,
    pub gyro_full_range: FsSel,
    pub accel_full_range: AfsSel,
    pub accel_x_raw: i16,
    pub accel_x: f64,
    pub accel_y_raw: i16,
    pub accel_y: f64,
    pub accel_z_raw: i16,
    pub accel_z: f64,
    pub gyro_x_raw: i16,
    pub gyro_x: f64,
    pub gyro_y_raw: i16,
    pub gyro_y: f64,
    pub gyro_z_raw: i16,
    pub gyro_z: f64,
    pub temperature: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CalibrationOffsets {
    pub accel_x: i16,
    pub accel_y: i16,
    pub accel_z: i16,
    pub gyro_x: i16,
    pub gyro_y: i16,
    pub gyro_z: i16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CalibrationProcess {
    /// 1 = запустить калибровку
    pub start: bool,

    /// 1 = калибровка запущена
    pub run: bool,

    /// Количество измерений, которые пропускаются перед началом калибровки
    pub skip_measurements: u16,

    /// Количество измерений, на основе которых рассчитывается смещение
    pub buffer_count: u16,

    /// Количество шагов калибровки
    pub steps: u8,

    pub current_measurement_in_step: u16,

    pub current_step: u8,

    pub buffer_accel_x: i64,
    pub buffer_accel_y: i64,
    pub buffer_accel_z: i64,
    pub buffer_gyro_x: i64,
    pub buffer_gyro_y: i64,
    pub buffer_gyro_z: i64,
    pub prev_offset_accel_x: i64,
    pub prev_offset_accel_y: i64,
    pub prev_offset_accel_z: i64,
    pub prev_offset_gyro_x: i64,
    pub prev_offset_gyro_y: i64,
    pub prev_offset_gyro_z: i64,

    pub gyro_full_range: FsSel,
    pub accel_full_range: AfsSel,
}
impl Default for CalibrationProcess {
    fn default() -> Self {
        Self {
            start: false,
            run: false,
            skip_measurements: 50,
            buffer_count: 50,
            steps: 5,
            current_measurement_in_step: 0,
            current_step: 0,
            buffer_accel_x: 0,
            buffer_accel_y: 0,
            buffer_accel_z: 0,
            buffer_gyro_x: 0,
            buffer_gyro_y: 0,
            buffer_gyro_z: 0,
            prev_offset_accel_x: 0,
            prev_offset_accel_y: 0,
            prev_offset_accel_z: 0,
            prev_offset_gyro_x: 0,
            prev_offset_gyro_y: 0,
            prev_offset_gyro_z: 0,
            gyro_full_range: Default::default(),
            accel_full_range: Default::default(),
        }
    }
}
