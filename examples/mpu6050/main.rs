mod config_linux_i2c_master;
mod config_timescaledb;
mod message;

use std::time::Duration;

use rsiot::{
    executor::{ComponentExecutor, ComponentExecutorConfig},
    logging::{LogConfig, LogConfigFilter},
};

#[tokio::main]
async fn main() {
    LogConfig {
        // filter: LogConfigFilter::String("info,rsiot::components::cmp_linux_i2c_master=trace"),
        filter: LogConfigFilter::String("info"),
    }
    .run()
    .unwrap();

    let config_executor = ComponentExecutorConfig {
        buffer_size: 100,
        fn_auth: |msg, _| Some(msg),
        delay_publish: Duration::from_millis(100),
        fn_tokio_metrics: |_| None,
    };

    ComponentExecutor::new(config_executor)
        .add_cmp(config_linux_i2c_master::cmp())
        .add_cmp(config_timescaledb::cmp())
        .wait_result()
        .await
        .unwrap();
}
