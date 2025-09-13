//! cargo build --example xpt2046_rpi --target="aarch64-unknown-linux-gnu" --release; scp target/aarch64-unknown-linux-gnu/release/examples/xpt2046_rpi user@target:/home/user/
//!
//! cross build --example xpt2046_rpi --target="aarch64-unknown-linux-gnu" --release; scp target/aarch64-unknown-linux-gnu/release/examples/xpt2046_rpi user@target:/home/user/

mod config_linux_spi_master;
mod messages;

use rsiot::components::cmp_logger;
use rsiot::executor::{ComponentExecutor, ComponentExecutorConfig};
use rsiot::logging::{LogConfig, LogConfigFilter};
use rsiot::message::Message;
use std::time::Duration;
use tracing::Level;

use messages::*;

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
        fn_input: |msg: Message<Msg>| {
            let Some(msg) = msg.get_custom_data() else {
                return Ok(None);
            };
            match msg {
                Msg::TouchEvent { x, y } => {
                    let s = format!("x: {}, y: {}", x, y);
                    Ok(Some(s))
                }
            }

            // Ok(Some(msg.serialize()?))
        },
    };

    // executor ------------------------------------------------------------------------------------
    let executor_config = ComponentExecutorConfig {
        buffer_size: 100,
        delay_publish: Duration::from_millis(100),
        fn_auth: |msg, _| Some(msg),
        fn_tokio_metrics: |_| None,
    };

    ComponentExecutor::<Msg>::new(executor_config)
        .add_cmp(cmp_logger::Cmp::new(config_logger))
        .add_cmp(config_linux_spi_master::cmp())
        .wait_result()
        .await
        .unwrap();
}
