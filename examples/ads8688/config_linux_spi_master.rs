use std::time::Duration;

use rsiot::{components::cmp_linux_spi_master::*, executor::Component};

use rsiot_devices::spi::ads8688;

use crate::messages::*;

pub fn cmp() -> Component<Config<Msg>, Msg> {
    let config = Config {
        devices_comm_settings: vec![ConfigDevicesCommSettings {
            linux_device: LinuxDevice::SpiWithCs {
                dev_spi: "/dev/spidev0.0".into(),
                dev_gpio: "/dev/gpiochip0".into(),
                gpio_line: 10,
            },
            baudrate: 1_000_000,
            spi_mode: ConfigDeviceSpiMode::Mode0,
        }],
        devices: vec![Box::new(ads8688::Device {
            request_period: Duration::from_millis(500),
            fn_output: |buffer| {
                if buffer.x == 0 {
                    return vec![];
                }

                vec![]
            },
        })],
    };

    Cmp::new(config)
}
