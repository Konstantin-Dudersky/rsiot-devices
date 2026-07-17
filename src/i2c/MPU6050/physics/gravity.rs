//! Gravity Vector Calculations
//!
//! The gravity vector represents the direction of gravitational acceleration
//! relative to the sensor's orientation. This is crucial for:
//! - Determining the sensor's orientation relative to Earth
//! - Separating gravity from linear acceleration
//! - Converting between quaternions and Euler angles
//!
//! The vector components indicate:
//! - x: Forward/backward tilt (pitch)
//! - y: Left/right tilt (roll)
//! - z: Vertical alignment (1.0 when level)

use super::quaternion::Quaternion;

/// A 3D vector representing the direction of gravity relative to the sensor.
///
/// Properties:
/// - When the sensor is perfectly level:
///   * x = 0 (no forward/backward tilt)
///   * y = 0 (no left/right tilt)
///   * z = 1 (gravity points straight down)
/// - The vector magnitude should always be approximately 1g
#[derive(Debug, Copy, Clone)]
pub struct Gravity {
    /// Forward/backward tilt component
    pub x: f32,
    /// Left/right tilt component
    pub y: f32,
    /// Vertical component
    pub z: f32,
}

impl From<Quaternion> for Gravity {
    /// Converts a quaternion to a gravity vector.
    ///
    /// This uses the quaternion rotation matrix to transform the unit vector
    /// [0, 0, 1] (representing gravity when level) to match the sensor's
    /// current orientation.
    ///
    /// The formulas used are derived from the quaternion rotation matrix:
    /// - x = 2(qx*qz - qw*qy)     [pitch component]
    /// - y = 2(qw*qx + qy*qz)     [roll component]
    /// - z = qw² - qx² - qy² + qz² [vertical component]
    fn from(q: Quaternion) -> Self {
        Self {
            x: 2.0 * (q.x * q.z - q.w * q.y),
            y: 2.0 * (q.w * q.x + q.y * q.z),
            z: q.w * q.w - q.x * q.x - q.y * q.y + q.z * q.z,
        }
    }
}
