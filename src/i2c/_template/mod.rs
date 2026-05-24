//! Шаблон драйвера устройства

mod buffer;
mod device;
mod output_data;
mod request_kind;

pub use {buffer::Buffer, device::Device, output_data::OutputData};

const DEVICE_NAME: &str = "DS3231";
