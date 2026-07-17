use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Выходные данные
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OutputData {
    pub accel_x: f64,
    pub accel_y: f64,
    pub accel_z: f64,
    pub gyro_x: f64,
    pub gyro_y: f64,
    pub gyro_z: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw: f64,
    pub temperature: f64,
}

impl Display for OutputData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "shunt_voltage: mV",)
    }
}
