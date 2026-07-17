//! Драйвера устройств для работы по шине I2C

#![allow(non_snake_case)]

pub mod DS3231;
pub mod MPU6050;
pub mod _template;
pub mod aht10;
pub mod as5600;

use async_trait::async_trait;
use bitvec::{field::BitField, order::Msb0, view::BitView};
use rsiot::{
    components_config::{
        i2c_master::{FieldbusRequest, FieldbusResponse, I2cAddress, Operation},
        master_device::{BufferBound, ConfigPeriodicRequest, DeviceBase, DeviceTrait, Result},
    },
    message::{Message, MsgDataBound},
};
use std::time::Duration;
use strum::FromRepr;
use tokio::sync::mpsc;

fn device_id(name: impl AsRef<str>, address: I2cAddress) -> String {
    format!("{} ({})", name.as_ref(), address)
}
