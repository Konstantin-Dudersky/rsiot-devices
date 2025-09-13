use serde::{Deserialize, Serialize};

use rsiot::message::{MsgDataBound, MsgKey};

#[derive(Clone, Debug, Deserialize, MsgKey, PartialEq, Serialize)]
pub enum Msg {
    MI2c(MI2c),
}

impl MsgDataBound for Msg {}

#[derive(Clone, Debug, Deserialize, MsgKey, PartialEq, Serialize)]
pub enum MI2c {
    Measurement {
        accel_x: f64,
        accel_y: f64,
        accel_z: f64,
        temperature: f64,
        gyro_x: f64,
        gyro_y: f64,
        gyro_z: f64,
    },
}
