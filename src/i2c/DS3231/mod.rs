//! Часы реального времени DS3231

mod buffer;
mod device;
mod ds3231_datetime;
mod output_data;
mod request_kind;

pub use {
    buffer::Buffer, device::Device, ds3231_datetime::Ds3231Datetime, output_data::OutputData,
};

const DEVICE_NAME: &str = "DS3231";
