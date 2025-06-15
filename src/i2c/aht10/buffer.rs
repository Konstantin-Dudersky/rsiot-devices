use super::BufferBound;

/// Буфер данных
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Buffer {
    pub address: u8,
    pub temperature: f64,
    pub humidity: f64,
}
impl BufferBound for Buffer {}
