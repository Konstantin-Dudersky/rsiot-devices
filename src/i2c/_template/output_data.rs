use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Выходные данные
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OutputData {}

impl Display for OutputData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "shunt_voltage: mV",)
    }
}
