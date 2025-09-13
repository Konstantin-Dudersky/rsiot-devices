use rsiot::message::{MsgDataBound, MsgKey};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, MsgKey, PartialEq, Serialize)]
pub enum Msg {
    TouchEvent { x: u32, y: u32 },
}

impl MsgDataBound for Msg {}
