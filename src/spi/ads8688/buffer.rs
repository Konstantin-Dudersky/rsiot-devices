use rsiot::components_config::master_device::BufferBound;

/// Буфер данных
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Buffer {
    pub x: u32,
    pub y: u32,
}

impl BufferBound for Buffer {}
