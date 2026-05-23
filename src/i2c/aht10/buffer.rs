use rsiot::components_config::i2c_master::I2cAddress;

use super::BufferBound;

/// Буфер данных
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Buffer {
    pub address: I2cAddress,
    pub temperature: f64,
    pub humidity: f64,
}
impl BufferBound for Buffer {}
