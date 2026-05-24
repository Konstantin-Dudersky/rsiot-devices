use rsiot::{
    components::shared_tasks::fieldbus_execution::FieldbusDiag,
    message::{MsgDataBound, MsgKey},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, MsgKey, PartialEq, Serialize)]
pub enum Msg {
    TouchEvent { x: u32, y: u32 },
    Diag(FieldbusDiag),
}

impl MsgDataBound for Msg {}
