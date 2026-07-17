use std::time::Duration;

use rsiot::{components::cmp_tsdb_writer::*, executor::Component};
use tracing::debug;

use super::msg::*;

pub fn cmp() -> Component<Config<Msg>, Msg> {
    let table = ConfigTable {
        cmp: "mpu".into(),
        tag: "mpu".into(),
        chunk_interval: Duration::from_hours(1),
        compress_interval: Duration::from_hours(1),
        retention_interval: Some(Duration::from_hours(24)),
        fields: vec![
            ConfigTableField {
                field_name: "accel_x".into(),
                data_type: ConfigTableFieldType::NumericDoublePrecision,
            },
            ConfigTableField {
                field_name: "accel_y".into(),
                data_type: ConfigTableFieldType::NumericDoublePrecision,
            },
            ConfigTableField {
                field_name: "accel_z".into(),
                data_type: ConfigTableFieldType::NumericDoublePrecision,
            },
            ConfigTableField {
                field_name: "gyro_x".into(),
                data_type: ConfigTableFieldType::NumericDoublePrecision,
            },
            ConfigTableField {
                field_name: "gyro_y".into(),
                data_type: ConfigTableFieldType::NumericDoublePrecision,
            },
            ConfigTableField {
                field_name: "gyro_z".into(),
                data_type: ConfigTableFieldType::NumericDoublePrecision,
            },
            ConfigTableField {
                field_name: "yaw".into(),
                data_type: ConfigTableFieldType::NumericDoublePrecision,
            },
            ConfigTableField {
                field_name: "pitch".into(),
                data_type: ConfigTableFieldType::NumericDoublePrecision,
            },
            ConfigTableField {
                field_name: "roll".into(),
                data_type: ConfigTableFieldType::NumericDoublePrecision,
            },
            ConfigTableField {
                field_name: "temperature".into(),
                data_type: ConfigTableFieldType::NumericDoublePrecision,
            },
        ],
        fn_input: |msg| {
            if let Msg::MI2c(MI2c::Measurement {
                accel_x,
                accel_y,
                accel_z,
                temperature,
                gyro_x,
                gyro_y,
                gyro_z,
                yaw,
                pitch,
                roll,
            }) = msg
            {
                let values = [
                    accel_x.to_string(),
                    accel_y.to_string(),
                    accel_z.to_string(),
                    gyro_x.to_string(),
                    gyro_y.to_string(),
                    gyro_z.to_string(),
                    yaw.to_string(),
                    pitch.to_string(),
                    roll.to_string(),
                    temperature.to_string(),
                ];
                let rows = row_without_ts(&values).unwrap();
                return Ok(Some(rows));
            }
            Ok(None)
        },
        delete_before_write: true,
    };

    let config = Config {
        connection_string: ConfigConnectionString {
            user: "postgres".into(),
            password: "postgres".into(),
            database_host: "192.168.0.2".into(),
            port: 5432,
            prj: "prj_rsiot_devices".into(),
            hst: "hst_local".into(),
            svc: "svc_mpu6050".into(),
        },
        max_connections: 5,
        tables: vec![table],
        save_by_row_count: 20_000,
        save_by_period: Duration::from_secs(2),
        fn_query_stat: |qs| debug!("{}", qs.to_string()),
    };

    Cmp::new(config)
}
