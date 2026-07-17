#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum ClockSource {
    /// Internal 8MHz oscillator
    /// - Fastest startup
    /// - Less accurate
    /// - Higher temperature drift
    Internal = 0,

    /// X-axis gyroscope reference
    /// - Recommended for general use
    /// - Good stability
    /// - Low temperature drift
    Xgyro = 1,

    /// Y-axis gyroscope reference
    /// - Alternative to X-axis
    /// - Similar stability to X-axis
    Ygyro = 2,

    /// Z-axis gyroscope reference
    /// - Alternative to X/Y-axis
    /// - Similar stability to X/Y-axis
    Zgyro = 3,

    /// External 32.768kHz crystal
    /// - Highest accuracy
    /// - Requires external crystal
    /// - Common RTC frequency
    PllExt32768 = 4,

    /// External 19.2MHz crystal
    /// - High accuracy
    /// - Requires external crystal
    /// - Typical system clock frequency
    External19200 = 5,

    /// Stops the clock
    /// - Lowest power consumption
    /// - Sensor stops operating
    /// - Must be restarted to resume
    Stop = 7,
}
