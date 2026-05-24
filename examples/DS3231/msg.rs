use rsiot_devices::i2c::DS3231;
use serde::{Deserialize, Serialize};

use rsiot::{
    components::shared_tasks::fieldbus_execution::FieldbusDiag,
    message::{MsgDataBound, MsgKey},
};

#[derive(Clone, Debug, Deserialize, MsgKey, PartialEq, Serialize)]
pub enum Msg {
    MsgI2c(MsgI2c),
    MsgInjectSingle(DS3231::Ds3231Datetime),
}

impl MsgDataBound for Msg {}

#[derive(Clone, Debug, Deserialize, MsgKey, PartialEq, Serialize)]
pub enum MsgI2c {
    Diag(FieldbusDiag),
    DS3231OutputData(DS3231::OutputData),
    DatetimeUtc(time::OffsetDateTime),
}
