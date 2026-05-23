use std::time::Duration;

use rsiot::{components::cmp_tsdb_reader::*, executor::Component};

use super::message::*;

pub fn cmp() -> Component<Config<Msg>, Msg> {
    let config = Config {
        connection_string: "postgres://postgres:postgres@dev:5432/db_data".into(),
        max_connections: 5,
        send_period: Duration::from_secs(1),
        fn_input: |msg| {
            let rows = match msg {
                Msg::MI2c(msg) => match msg {
                    MI2c::Measurement {
                        accel_x,
                        accel_y,
                        accel_z,
                        temperature,
                        gyro_x,
                        gyro_y,
                        gyro_z,
                    } => {
                        vec![
                            Row::new_simple("accelerometer", "accel_x", *accel_x),
                            Row::new_simple("accelerometer", "accel_y", *accel_y),
                            Row::new_simple("accelerometer", "accel_z", *accel_z),
                            Row::new_simple("accelerometer", "temperature", *temperature),
                            Row::new_simple("accelerometer", "gyro_x", *gyro_x),
                            Row::new_simple("accelerometer", "gyro_y", *gyro_y),
                            Row::new_simple("accelerometer", "gyro_z", *gyro_z),
                        ]
                    }
                },
            };
            Some(rows)
        },
        table_name: "raw",
        delete_before_write: false,
    };

    Cmp::new(config)
}
