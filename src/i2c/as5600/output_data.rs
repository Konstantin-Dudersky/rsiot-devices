use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OutputData {
    pub angle_raw: f64,
    pub angle: f64,
    pub agc: u8,
    pub magnitude: u16,
    pub status_magnet_detected: bool,
    pub status_magnet_too_strong: bool,
    pub status_magnet_too_weak: bool,
}
