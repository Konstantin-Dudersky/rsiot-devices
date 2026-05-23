use std::time::Duration;

use rsiot::{components::cmp_linux_i2c_master::*, executor::Component};
use rsiot_devices::i2c::mpu6050;

use super::message::*;

pub fn cmp() -> Component<Config<Msg>, Msg> {
    let config = Config::<Msg> {
        dev_i2c: "/dev/i2c-0".into(),
        devices: vec![Box::new(mpu6050::Device {
            address: I2cAddress::Mux {
                mux_address: 0x71,
                channel: 0b0100_0000,
                address: 0x68,
            },
            request_period: Duration::from_millis(10),
            fn_output: |buffer| {
                vec![Msg::MI2c(MI2c::Measurement {
                    accel_x: buffer.read_data.accel_x,
                    accel_y: buffer.read_data.accel_y,
                    accel_z: buffer.read_data.accel_z,
                    temperature: buffer.read_data.temperature,
                    gyro_x: buffer.read_data.gyro_x,
                    gyro_y: buffer.read_data.gyro_y,
                    gyro_z: buffer.read_data.gyro_z,
                })]
            },
            gyro_full_range: mpu6050::FsSel::_1000DPS,
            accel_full_range: mpu6050::AfsSel::_2G,
            default_calibration_offset_accel_x: -4232,
            default_calibration_offset_accel_y: 2864,
            default_calibration_offset_accel_z: 667,
            default_calibration_offset_gyro_x: -30,
            default_calibration_offset_gyro_y: -54,
            default_calibration_offset_gyro_z: 40,
            default_calibration_start: false,
        })],
    };

    Cmp::new(config)
}
