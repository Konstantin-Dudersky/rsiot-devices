//! MPU-6050 - датчик ускорения и угловой скорости
//!
//! # Калибровка
//!
//! Датчик калибруется на заводе. Калибровочные смещения сохраняются в регистрах. Смещения можно
//! изменить, но после перезапуска опять будут возвращены заводские значения.
//!
//! При запуске заводские параметры калибровки считываются и выводятся в консоли. После этого в
//! датчик записываются данные из настройки устройства из полей default_calibration_offset_XXX.
//! Поэтому для использования заводских значений, нужно скопировать значения из консоли и вставить в
//! конфигурацию устройства.

mod buffer;
mod device;
mod outline_detection;
mod output_data;
mod physics;
mod registers;
mod request_kind;

pub use {
    buffer::Buffer,
    device::Device,
    output_data::OutputData,
    physics::{AccelFullScale, GyroFullScale},
};

use request_kind::RequestKind;

use super::{
    async_trait, mpsc, BitField, BitView, BufferBound, ConfigPeriodicRequest, DeviceBase,
    DeviceTrait, Duration, FieldbusRequest, FieldbusResponse, FromRepr, Message, Msb0,
    MsgDataBound, Operation, Result,
};

const DEVICE_NAME: &str = "MPU6050";
