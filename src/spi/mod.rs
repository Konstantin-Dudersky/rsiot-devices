pub mod xpt2046;

use async_trait::async_trait;
use rsiot::{
    components_config::{
        master_device::{BufferBound, ConfigPeriodicRequest, DeviceBase, DeviceTrait, Result},
        spi_master::{FieldbusRequest, FieldbusResponse, Operation},
    },
    message::{Message, MsgDataBound},
};
use tokio::sync::{broadcast, mpsc};
