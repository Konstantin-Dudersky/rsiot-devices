use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::Ds3231Datetime;

/// Выходные данные
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OutputData {
    /// Время
    pub datetime: Ds3231Datetime,

    /// Температура
    pub temperature: f32,
}

impl Display for OutputData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Datetime: {}; temperature: {:.2} ℃",
            self.datetime, self.temperature
        )
    }
}
