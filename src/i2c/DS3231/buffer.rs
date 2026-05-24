use rsiot::components_config::{i2c_master::I2cAddress, master_device::BufferBound};

use super::{Ds3231Datetime, OutputData};

/// Буфер данных
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Buffer {
    /// Конфигурация
    pub config: Config,

    /// Данные для записи
    pub write_data: WriteData,

    /// Прочитанные данные
    pub read_data: ReadData,
}

impl BufferBound for Buffer {}

impl Buffer {
    /// Создать выходные данные
    pub fn output_data(&self) -> OutputData {
        OutputData {
            datetime: self.read_data.datetime.clone(),
            temperature: self.read_data.temperature,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Config {
    pub address: I2cAddress,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WriteData {
    pub need_write: bool,
    pub datetime: Ds3231Datetime,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReadData {
    pub datetime: Ds3231Datetime,
    pub temperature: f32,
}
