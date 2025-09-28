pub mod ads8688;
pub mod xpt2046;

use async_trait::async_trait;
use rsiot::{
    components_config::{
        master_device::{BufferBound, ConfigPeriodicRequest, DeviceBase, DeviceTrait},
        spi_master::{FieldbusRequest, FieldbusResponse, Operation},
    },
    message::{Message, MsgDataBound},
};
use std::time::Duration;
use tokio::sync::mpsc;
