use std::time::Duration;

use rsiot::{components::cmp_linux_i2c_master::*, executor::Component};
use rsiot_devices::i2c::MPU6050;
use tracing::info;

use super::msg::*;

pub fn cmp() -> Component<Config<Msg>, Msg> {
    let config = Config::<Msg> {
        dev_i2c: "/dev/i2c-0".into(),
        devices: vec![Box::new(MPU6050::Device {
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
                    yaw: buffer.read_data.yaw,
                    pitch: buffer.read_data.pitch,
                    roll: buffer.read_data.roll,
                })]
            },
            gyro_full_range: MPU6050::GyroFullScale::Deg1000,
            accel_full_range: MPU6050::AccelFullScale::G2,
            calibration_accel_x: -501,
            calibration_accel_y: -3861,
            calibration_accel_z: 1269,
            calibration_gyro_x: 104,
            calibration_gyro_y: 2,
            calibration_gyro_z: 52,
            start_calibration: false,
            dmp_enabled: true,
        })],
        fn_diag: |diag| {
            info!("I2C diag: {:?}", diag);
            Msg::MI2c(MI2c::Diag(diag.clone()))
        },
        fn_diag_period: Duration::from_millis(1_000),
    };

    Cmp::new(config)
}
