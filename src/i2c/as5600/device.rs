use std::time::Duration;

use async_trait::async_trait;
use bitvec::{field::BitField, order::Msb0, view::BitView};
use rsiot::{
    components_config::{
        i2c_master::{FieldbusRequest, FieldbusResponse, I2cAddress, Operation},
        master_device::{
            ConfigDeviceStateOutput, ConfigPeriodicRequest, DeviceBase, DeviceTrait,
            ResponseResult, Result,
        },
    },
    executor::MsgBusInput,
    message::{Message, MsgDataBound},
};
use tokio::sync::mpsc;
use tracing::trace;

use super::{
    buffer::{Config, WriteData},
    {request_kind::RequestKind, Buffer},
};

/// Датчик температуры и влажности AHT10
#[derive(Clone, Debug)]
pub struct Device<TMsg>
where
    TMsg: MsgDataBound,
{
    /// Адрес чипа фиксирован: 0x36
    pub address: I2cAddress,

    pub request_period: Duration,

    /// Преобразование данных из буфера в исходящие сообщения
    pub fn_output: fn(&mut Buffer) -> Vec<TMsg>,

    pub device_state_output: ConfigDeviceStateOutput<TMsg>,
}

#[async_trait]
impl<TMsg> DeviceTrait<TMsg, FieldbusRequest, FieldbusResponse> for Device<TMsg>
where
    TMsg: MsgDataBound + 'static,
{
    async fn spawn(
        self: Box<Self>,
        ch_rx_msgbus_to_device: MsgBusInput<TMsg>,
        ch_tx_device_to_fieldbus: mpsc::Sender<FieldbusRequest>,
        ch_rx_fieldbus_to_device: mpsc::Receiver<FieldbusResponse>,
        ch_tx_device_to_msgbus: mpsc::Sender<Message<TMsg>>,
    ) -> Result<()> {
        let periodic_requests = vec![ConfigPeriodicRequest {
            period: self.request_period,
            fn_requests: |buffer: &Buffer| {
                let mut requests = vec![];

                let req = FieldbusRequest::new(
                    buffer.config.address,
                    RequestKind::ReadStatusAndOutput,
                    vec![
                        Operation::WriteRead {
                            write_data: vec![0x0B],
                            read_size: 5,
                        },
                        Operation::WriteRead {
                            write_data: vec![0x1A],
                            read_size: 3,
                        },
                    ],
                );
                requests.push(req);

                Ok(requests)
            },
        }];

        let buffer_default = Buffer {
            config: Config {
                address: self.address,
            },
            write_data: WriteData::default(),
            ..Default::default()
        };

        let device = DeviceBase {
            fn_init_requests,
            periodic_requests,
            fn_msgs_to_buffer: |_msg, _buffer| (),
            buffer_to_request_period: Duration::from_millis(1000),
            fn_buffer_to_request,
            fn_response_to_buffer,
            fn_buffer_to_msgs: self.fn_output,
            device_state_output: Some(self.device_state_output),
            buffer_default,
        };
        device
            .spawn(
                "AS5600",
                ch_rx_msgbus_to_device,
                ch_tx_device_to_fieldbus,
                ch_rx_fieldbus_to_device,
                ch_tx_device_to_msgbus,
            )
            .await?;
        Ok(())
    }
}

fn fn_init_requests(buffer: &Buffer) -> Vec<FieldbusRequest> {
    let mut requests = vec![];

    let req = FieldbusRequest::new(buffer.config.address, RequestKind::Init, vec![]);
    requests.push(req);

    requests
}

fn fn_buffer_to_request(_buffer: &Buffer) -> anyhow::Result<Vec<FieldbusRequest>> {
    let mut _requests = vec![];

    Ok(_requests)
}

fn fn_response_to_buffer(
    response: FieldbusResponse,
    buffer: &mut Buffer,
) -> anyhow::Result<ResponseResult> {
    trace!("Response: {:?}", response);

    let request_kind: RequestKind = response.request_kind.try_into()?;

    let payload = match response.payload {
        Ok(payload) => payload,
        Err(err) => {
            return ResponseResult::error(err);
        }
    };

    match request_kind {
        RequestKind::Init => ResponseResult::ok_init_completed(),

        RequestKind::ReadStatusAndOutput => {
            // 0 - статус и выходные значения ------------------------------------------------------
            let bits = payload[0].view_bits::<Msb0>();

            buffer.read_data.status_magnet_detected = bits[2];
            buffer.read_data.status_magnet_too_weak = bits[3];
            buffer.read_data.status_magnet_too_strong = bits[4];

            let angle_raw: u16 = bits[12..24].load_be();
            buffer.read_data.angle_raw = (angle_raw as f64 * 360.0) / 4096.0;

            let angle: u16 = bits[28..40].load_be();
            buffer.read_data.angle = (angle as f64 * 360.0) / 4096.0;

            // 1 - AGC и magnitude -----------------------------------------------------------------
            let bits = payload[1].view_bits::<Msb0>();

            let agc: u8 = bits[0..8].load_be();
            buffer.read_data.agc = agc;

            let magnitude: u16 = bits[12..24].load_be();
            buffer.read_data.magnitude = magnitude;

            ResponseResult::ok()
        }
    }
}
