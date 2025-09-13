use std::time::Duration;

use async_trait::async_trait;
use rsiot::{
    components_config::{
        master_device::{ConfigPeriodicRequest, DeviceBase, DeviceTrait, Result},
        spi_master::{FieldbusRequest, FieldbusResponse, Operation},
    },
    message::{Message, MsgDataBound},
};
use tokio::sync::{broadcast, mpsc};
use tracing::{info, trace};

use super::{request_kind::RequestKind, Buffer};

/// Тестовое устройство
#[derive(Clone, Debug)]
pub struct Device<TMsg> {
    pub request_period: Duration,

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
        ch_rx_msgbus_to_device: broadcast::Receiver<Message<TMsg>>,
        ch_tx_device_to_fieldbus: mpsc::Sender<FieldbusRequest>,
        ch_rx_fieldbus_to_device: mpsc::Receiver<FieldbusResponse>,
        ch_tx_device_to_msgbus: mpsc::Sender<Message<TMsg>>,
    ) -> Result<()> {
        let device = DeviceBase {
            fn_init_requests: |_| {
                vec![FieldbusRequest::new(
                    RequestKind::SetCommandAutoRst,
                    vec![
                        Operation::Write(vec![0x85, 0x00]),
                        Operation::Write(vec![0xA0, 0x00]),
                    ],
                )]
            },
            periodic_requests: vec![ConfigPeriodicRequest {
                period: self.request_period,
                fn_requests: |_buffer| {
                    Ok(vec![FieldbusRequest::new(
                        RequestKind::ReadAutoSeq,
                        vec![
                            Operation::WriteRead(vec![0x00, 0x00, 0x00], 3),
                            // Operation::Read { read_size: 1 },
                        ],
                    )])
                },
            }],
            fn_msgs_to_buffer: |_msg, _buffer| (),
            buffer_to_request_period: Duration::from_millis(100),
            fn_buffer_to_request: |_buffer: &Buffer| Ok(vec![]),
            fn_response_to_buffer: |response: FieldbusResponse, buffer: &mut Buffer| {
                info!("Response: {:?}", response);

                let request_kind: RequestKind = response.request_kind.into();

                // match request_kind {
                //     RequestKind::XYPosition => {
                //         let response_x = response.payload[0][0];
                //         let response_y = response.payload[1][0];

                //         if response_x == 0 {
                //             buffer.x = 0;
                //             buffer.y = 0;
                //         } else {
                //             buffer.x = response_x as u32;
                //             buffer.y = response_y as u32;
                //         }
                //     }
                // }

                Ok(false)
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
