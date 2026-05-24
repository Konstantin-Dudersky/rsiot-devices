use std::time::Duration;

use async_trait::async_trait;
use rsiot::{
    components_config::{
        i2c_master::{FieldbusRequest, FieldbusResponse, I2cAddress, Operation},
        master_device::{
            ConfigPeriodicRequest, DeviceBase, DeviceTrait, FieldbusDiagMsg, ResponseResult, Result,
        },
    },
    executor::MsgBusInput,
    message::{Message, MsgDataBound},
};
use tokio::sync::mpsc;
use tracing::{trace, warn};

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
    /// Адрес зависит от AD0:
    /// - GND - 0x68
    /// - VCC - 0x69
    pub address: I2cAddress,

    /// Период чтения данных
    pub request_period: Duration,

    /// Преобразование данных из буфера в исходящие сообщения
    pub fn_input: fn(&TMsg, &mut Buffer) -> (),

    /// Преобразование данных из буфера в исходящие сообщения
    pub fn_output: fn(&mut Buffer) -> Vec<TMsg>,
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
        ch_tx_device_to_diag: mpsc::Sender<FieldbusDiagMsg>,
    ) -> Result<()> {
        let periodic_requests = vec![ConfigPeriodicRequest {
            period: self.request_period,
            fn_requests: |buffer: &Buffer| {
                let mut requests = vec![];

                let req = FieldbusRequest::new(
                    buffer.config.address,
                    RequestKind::ReadValues,
                    vec![Operation::WriteRead {
                        write_data: vec![0x3B],
                        read_size: 14,
                    }],
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
            fn_msgs_to_buffer: self.fn_input,
            buffer_to_request_period: Duration::from_millis(1000),
            fn_buffer_to_request,
            fn_response_to_buffer,
            fn_buffer_to_msgs: self.fn_output,
            buffer_default,
        };
        device
            .spawn(
                super::super::device_id(super::DEVICE_NAME, self.address),
                ch_rx_msgbus_to_device,
                ch_tx_device_to_fieldbus,
                ch_rx_fieldbus_to_device,
                ch_tx_device_to_msgbus,
                ch_tx_device_to_diag,
            )
            .await?;
        Ok(())
    }
}

fn fn_init_requests(buffer: &Buffer) -> Vec<FieldbusRequest> {
    let mut requests = vec![];

    let req = FieldbusRequest::new(
        buffer.config.address,
        RequestKind::Init,
        vec![Operation::Write {
            write_data: vec![0x6B, 0x00],
        }],
    );
    requests.push(req);

    requests
}

fn fn_buffer_to_request(_buffer: &Buffer) -> anyhow::Result<Vec<FieldbusRequest>> {
    let mut _requests = vec![];

    Ok(_requests)
}

fn fn_response_to_buffer(
    response: FieldbusResponse,
    _buffer: &mut Buffer,
) -> anyhow::Result<ResponseResult> {
    trace!("Response: {:?}", response);

    let request_kind: RequestKind = response.request_kind.try_into()?;

    let _payload = match response.payload {
        Ok(payload) => payload,
        Err(err) => {
            warn!("Error reading MPU-6050: {}", err);
            return ResponseResult::error(err);
        }
    };

    match request_kind {
        RequestKind::Init => ResponseResult::ok_init_completed(),

        RequestKind::ReadValues => ResponseResult::ok(),
    }
}
