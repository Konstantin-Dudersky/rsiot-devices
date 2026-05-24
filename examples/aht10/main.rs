//! cargo build --example aht10 --target="armv7-unknown-linux-gnueabihf" --release; scp target/armv7-unknown-linux-gnueabihf/release/examples/aht10 root@target:/root

use rsiot::{
    components_config::i2c_master::I2cAddress,
    logging::{LogConfig, LogConfigFilter},
};
use rsiot_devices::i2c::aht10;
use tracing::info;

mod msg;

#[tokio::main]
async fn main() {
    LogConfig {
        filter: LogConfigFilter::String("info,rsiot::components::cmp_linux_i2c_master=trace"),
    }
    .run()
    .unwrap();

    use std::time::Duration;

    use rsiot::{
        components::cmp_linux_i2c_master,
        executor::{ComponentExecutor, ComponentExecutorConfig},
    };

    let config = cmp_linux_i2c_master::Config::<msg::Custom> {
        dev_i2c: "/dev/i2c-2".into(),
        devices: vec![Box::new(aht10::Device {
            address: I2cAddress::Direct { address: 0x38 },
            request_period: Duration::from_millis(1000),
            fn_output: |buffer| {
                info!("Humidity: {:.1}%", buffer.humidity);
                info!("Temperature: {:.1}°C", buffer.temperature);
                vec![]
            },
        })],
        fn_diag: |diag| msg::Custom::Diag(diag.clone()),
        fn_diag_period: Duration::from_millis(1_000),
    };

    let config_executor = ComponentExecutorConfig {
        buffer_size: 100,
        fn_auth: |msg, _| Some(msg),
        delay_publish: Duration::from_millis(100),
        fn_tokio_metrics: |_| None,
    };

    ComponentExecutor::new(config_executor)
        .add_cmp(cmp_linux_i2c_master::Cmp::new(config))
        .wait_result()
        .await
        .unwrap();
}
