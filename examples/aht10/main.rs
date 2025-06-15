//! cargo build --example aht10 --target="armv7-unknown-linux-gnueabihf" --release; scp target/armv7-unknown-linux-gnueabihf/release/examples/aht10 root@target:/root

use rsiot::logging::configure_logging;
use rsiot_devices::i2c::aht10;
use tracing::info;

mod message;

#[tokio::main]
async fn main() {
    configure_logging("info,rsiot::components::cmp_linux_i2c_master=trace", None)
        .await
        .unwrap();

    use std::time::Duration;

    use rsiot::{
        components::cmp_linux_i2c_master,
        executor::{ComponentExecutor, ComponentExecutorConfig},
    };

    let config = cmp_linux_i2c_master::Config::<message::Custom> {
        dev_i2c: "/dev/i2c-2".into(),
        devices: vec![Box::new(aht10::Device {
            address: 0x38,
            request_period: Duration::from_millis(1000),
            fn_output: |buffer| {
                info!("Humidity: {:.1}%", buffer.humidity);
                info!("Temperature: {:.1}°C", buffer.temperature);
                vec![]
            },
        })],
    };

    let config_executor = ComponentExecutorConfig {
        buffer_size: 100,
        fn_auth: |msg, _| Some(msg),
        delay_publish: Duration::from_millis(100),
    };

    ComponentExecutor::new(config_executor)
        .add_cmp(cmp_linux_i2c_master::Cmp::new(config))
        .wait_result()
        .await
        .unwrap();
}
