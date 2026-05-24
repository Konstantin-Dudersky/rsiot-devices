use rsiot::components::cmp_inject_single::*;
use rsiot_devices::i2c::DS3231::Ds3231Datetime;

use crate::msg::*;

pub fn cmp() -> Cmp<Msg, impl FnOnce() -> Vec<Msg>> {
    let time_now = Ds3231Datetime::now_from_crate_time();

    let config = Config {
        fn_output: move || vec![Msg::MsgInjectSingle(time_now)],
    };

    Cmp::new(config)
}
