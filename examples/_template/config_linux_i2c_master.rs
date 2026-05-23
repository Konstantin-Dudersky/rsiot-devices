use std::time::Duration;

use rsiot::{
    components::cmp_linux_i2c_master::*, components_config::master_device::ConfigDeviceStateOutput,
};
use rsiot_devices::i2c::_template;

use super::message::*;

pub fn cmp() -> Cmp<Msg> {
    let config = Config::<Msg> {
        dev_i2c: "/dev/i2c-0".into(),
        devices: vec![Box::new(_template::Device {
            address: I2cAddress::Mux {
                mux_address: 0x71,
                channel: 0b0100_0000,
                address: 0x68,
            },
            request_period: Duration::from_millis(10),
            fn_output: |_buffer| vec![Msg::MsgI2c(MsgI2c::Meas(10))],
            device_state_output: ConfigDeviceStateOutput {
                fn_device_state: |ds| Msg::MsgI2c(MsgI2c::AS5600State(ds)),
                period: Duration::from_millis(1_000),
            },
        })],
    };

    Cmp::new(config)
}
