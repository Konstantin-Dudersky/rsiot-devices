use rsiot::components_config::{i2c_master::I2cAddress, master_device::BufferBound};

use super::OutputData;

/// Буфер данных
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Buffer {
    pub config: Config,
    pub write_data: WriteData,
    pub read_data: ReadData,
}

impl BufferBound for Buffer {}

impl Buffer {
    pub fn output_data(&self) -> OutputData {
        OutputData {}
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Config {
    pub address: I2cAddress,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WriteData {}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReadData {}
