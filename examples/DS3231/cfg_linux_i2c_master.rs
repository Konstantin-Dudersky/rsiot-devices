use std::time::Duration;

use rsiot::components::cmp_linux_i2c_master::*;
use rsiot_devices::i2c::DS3231;
use tracing::warn;

use super::msg::*;

pub fn cmp() -> Cmp<Msg> {
    let config = Config::<Msg> {
        dev_i2c: "/dev/i2c-1".into(),
        devices: vec![Box::new(DS3231::Device {
            address: I2cAddress::Direct { address: 0x68 },
            request_period: Duration::from_millis(1000),
            fn_input: |msg, buffer| {
                if let Msg::MsgInjectSingle(time_now) = msg {
                    buffer.write_data.need_write = true;
                    buffer.write_data.datetime = time_now.clone();
                }
            },
            fn_output: |buffer| {
                let mut msgs = vec![];

                let output_data = buffer.output_data();
                msgs.push(Msg::MsgI2c(MsgI2c::DS3231OutputData(buffer.output_data())));

                match output_data.datetime.into_crate_time() {
                    Ok(time_utc) => {
                        msgs.push(Msg::MsgI2c(MsgI2c::DatetimeUtc(time_utc)));
                    }
                    Err(_) => {
                        warn!("Error parsing datetime: {}", output_data.datetime);
                    }
                };

                msgs
            },
        })],
        fn_diag: |diag| Msg::MsgI2c(MsgI2c::Diag(diag.clone())),
        fn_diag_period: Duration::from_millis(1_000),
    };

    Cmp::new(config)
}
