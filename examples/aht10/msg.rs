use serde::{Deserialize, Serialize};

use rsiot::{
    components::shared_tasks::fieldbus_execution::FieldbusDiag,
    message::{MsgDataBound, MsgKey},
};

#[derive(Clone, Debug, Deserialize, MsgKey, PartialEq, Serialize)]
pub enum Custom {
    Counter(i32),
    Diag(FieldbusDiag),
}

impl MsgDataBound for Custom {}
