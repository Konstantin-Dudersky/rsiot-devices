use std::time::Duration;

use strum::FromRepr;
use tracing::trace;

use super::{
    async_trait, broadcast, mpsc, BufferBound, ConfigPeriodicRequest, DeviceBase, DeviceTrait,
    FieldbusRequest, FieldbusResponse, Message, MsgDataBound, Operation, Result,
};

/// Тестовое устройство
#[derive(Clone, Debug)]
pub struct Device<TMsg> {
    pub request_period: Duration,

    /// Преобразование данных из буфера в исходящие сообщения
    pub fn_output: fn(&mut Buffer) -> Vec<Message<TMsg>>,
}

#[async_trait]
impl<TMsg> DeviceTrait<TMsg, FieldbusRequest, FieldbusResponse> for Device<TMsg>
where
    TMsg: MsgDataBound + 'static,
{
    async fn spawn(
        self: Box<Self>,
        ch_rx_msgbus_to_device: broadcast::Receiver<Message<TMsg>>,
        ch_tx_device_to_fieldbus: mpsc::Sender<FieldbusRequest>,
        ch_rx_fieldbus_to_device: mpsc::Receiver<FieldbusResponse>,
        ch_tx_device_to_msgbus: mpsc::Sender<Message<TMsg>>,
    ) -> Result<()> {
        let device = DeviceBase {
            fn_init_requests: |_| vec![],
            periodic_requests: vec![ConfigPeriodicRequest {
                period: self.request_period,
                fn_requests: |_buffer| {
                    Ok(vec![FieldbusRequest::new(
                        RequestKind::XYPosition,
                        vec![
                            Operation::WriteRead(vec![0b1101_1000], 1),
                            Operation::WriteRead(vec![0b1001_1000], 1),
                        ],
                    )])
                },
            }],
            fn_msgs_to_buffer: |_msg, _buffer| (),
            fn_buffer_to_request: |_buffer: &Buffer| Ok(vec![]),
            fn_response_to_buffer: |response: FieldbusResponse, buffer: &mut Buffer| {
                trace!("Response: {:?}", response);

                let request_kind: RequestKind = response.request_kind.into();

                match request_kind {
                    RequestKind::XYPosition => {
                        let response_x = response.payload[0][0];
                        let response_y = response.payload[1][0];

                        if response_x == 0 {
                            buffer.x = 0;
                            buffer.y = 0;
                        } else {
                            buffer.x = response_x as u32;
                            buffer.y = response_y as u32;
                        }
                    }
                }

                Ok(())
            },
            fn_buffer_to_msgs: self.fn_output,
            buffer_default: Buffer::default(),
        };
        device
            .spawn(
                ch_rx_msgbus_to_device,
                ch_tx_device_to_fieldbus,
                ch_rx_fieldbus_to_device,
                ch_tx_device_to_msgbus,
            )
            .await?;
        Ok(())
    }
}

/// Виды запросов
#[derive(FromRepr)]
pub enum RequestKind {
    XYPosition,
}

impl From<RequestKind> for u8 {
    fn from(value: RequestKind) -> Self {
        value as u8
    }
}

impl From<u8> for RequestKind {
    fn from(value: u8) -> Self {
        RequestKind::from_repr(value as usize).unwrap()
    }
}

/// Буфер данных
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Buffer {
    pub x: u32,
    pub y: u32,
}

impl BufferBound for Buffer {}
