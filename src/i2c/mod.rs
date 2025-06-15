pub mod aht10;

use async_trait::async_trait;
use bitvec::{field::BitField, order::Msb0, view::BitView};
use rsiot::{
    components_config::{
        i2c_master::{FieldbusRequest, FieldbusResponse, Operation},
        master_device::{BufferBound, ConfigPeriodicRequest, DeviceBase, DeviceTrait, Result},
    },
    message::{Message, MsgDataBound},
};
use std::time::Duration;
use strum::FromRepr;
use tokio::sync::{broadcast, mpsc};
