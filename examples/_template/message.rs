use serde::{Deserialize, Serialize};

use rsiot::{
    components_config::master_device::DeviceState,
    message::{MsgDataBound, MsgKey},
};

#[derive(Clone, Debug, Deserialize, MsgKey, PartialEq, Serialize)]
pub enum Msg {
    MsgI2c(MsgI2c),
}

impl MsgDataBound for Msg {}

#[derive(Clone, Debug, Deserialize, MsgKey, PartialEq, Serialize)]
pub enum MsgI2c {
    Meas(u32),
    AS5600State(DeviceState),
}
