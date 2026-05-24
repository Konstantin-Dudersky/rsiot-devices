//! cargo build --example xpt2046_rpi --target="aarch64-unknown-linux-gnu" --release; scp target/aarch64-unknown-linux-gnu/release/examples/xpt2046_rpi user@target:/home/user/
//!
//! cross build --example xpt2046_rpi --target="aarch64-unknown-linux-gnu" --release; scp target/aarch64-unknown-linux-gnu/release/examples/xpt2046_rpi user@target:/home/user/

use rsiot::components::cmp_linux_spi_master::LinuxDevice;
use rsiot::components::shared_tasks::fieldbus_execution::FieldbusDiag;
use rsiot::components::{cmp_linux_spi_master, cmp_logger};
use rsiot::executor::{ComponentExecutor, ComponentExecutorConfig};
use rsiot::logging::{LogConfig, LogConfigFilter};
use rsiot::message::{Message, MsgDataBound, MsgKey};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::Level;

use rsiot_devices::spi::xpt2046::Device;

#[tokio::main]
async fn main() {
    LogConfig {
        filter: LogConfigFilter::String("info"),
    }
    .run()
    .unwrap();

    // cmp_logger ----------------------------------------------------------------------------------
    let config_logger = cmp_logger::Config {
        level: Level::INFO,
        fn_input: |msg: Message<Custom>| {
            let Some(msg) = msg.get_custom_data() else {
                return Ok(None);
            };
            match msg {
                Custom::TouchEvent { x, y } => {
                    let s = format!("x: {}, y: {}", x, y);
                    Ok(Some(s))
                }
                _ => Ok(None),
            }
        },
    };

    // cmp_linux_spi_master ------------------------------------------------------------------------
    let config_linux_spi_master = cmp_linux_spi_master::Config {
        devices_comm_settings: vec![cmp_linux_spi_master::ConfigDevicesCommSettings {
            linux_device: LinuxDevice::Spi {
                dev_spi: "/dev/spidev0.0".into(),
            },
            baudrate: 100_000,
            spi_mode: cmp_linux_spi_master::ConfigDeviceSpiMode::Mode0,
        }],
        devices: vec![Box::new(Device {
            request_period: Duration::from_millis(10),
            fn_output: |buffer| {
                if buffer.x == 0 {
                    return vec![];
                }
                let msg = Custom::TouchEvent {
                    x: buffer.x,
                    y: buffer.y,
                };
                vec![msg]
            },
        })],
        fn_diag: |diag| Custom::Diag(diag.clone()),
        fn_diag_period: Duration::from_millis(1_000),
    };

    // executor ------------------------------------------------------------------------------------
    let executor_config = ComponentExecutorConfig {
        buffer_size: 100,
        delay_publish: Duration::from_millis(100),
        fn_auth: |msg, _| Some(msg),
        fn_tokio_metrics: |_| None,
    };

    ComponentExecutor::<Custom>::new(executor_config)
        .add_cmp(cmp_logger::Cmp::new(config_logger))
        .add_cmp(cmp_linux_spi_master::Cmp::new(config_linux_spi_master))
        .wait_result()
        .await
        .unwrap();
}

#[derive(Clone, Debug, Deserialize, MsgKey, PartialEq, Serialize)]
enum Custom {
    TouchEvent { x: u32, y: u32 },
    Diag(FieldbusDiag),
}

impl MsgDataBound for Custom {}
