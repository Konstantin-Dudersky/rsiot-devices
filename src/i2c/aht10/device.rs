use tracing::{trace, warn};

use super::{
    async_trait, broadcast, mpsc, BitField, BitView, Buffer, ConfigPeriodicRequest, DeviceBase,
    DeviceTrait, Duration, FieldbusRequest, FieldbusResponse, Message, Msb0, MsgDataBound,
    Operation, RequestKind, Result,
};

/// Датчик температуры и влажности AHT10
#[derive(Clone, Debug)]
pub struct Device<TMsg> {
    pub address: u8,

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
            fn_init_requests: |_| vec![],
            periodic_requests: vec![ConfigPeriodicRequest {
                period: self.request_period,
                fn_requests: |buffer: &Buffer| {
                    Ok(vec![FieldbusRequest::new(
                        buffer.address,
                        RequestKind::Values,
                        vec![
                            Operation::Write {
                                write_data: vec![0xAC, 0x33, 0x00],
                            },
                            Operation::Delay {
                                delay: Duration::from_millis(100),
                            },
                            Operation::Read { read_size: 6 },
                        ],
                    )])
                },
            }],
            fn_msgs_to_buffer: |_msg, _buffer| (),
            fn_buffer_to_request: |_buffer: &Buffer| Ok(vec![]),
            fn_response_to_buffer: |response: FieldbusResponse, buffer: &mut Buffer| {
                trace!("Response: {:?}", response);

                let request_kind: RequestKind = response.request_kind.into();

                let payload = match response.payload {
                    Ok(payload) => payload,
                    Err(err) => {
                        warn!("Error reading AHT10: {}", err);
                        return Ok(());
                    }
                };

                match request_kind {
                    RequestKind::Values => {
                        let bits = payload[0].view_bits::<Msb0>();

                        let hum = bits[8..28].load_be::<u32>();
                        buffer.humidity = hum as f64 / 2_u32.pow(20) as f64 * 100.0;

                        let temp = bits[28..48].load_be::<u32>();
                        buffer.temperature = temp as f64 / 2_u32.pow(20) as f64 * 200.0 - 50.0;
                    }
                }

                Ok(())
            },
            fn_buffer_to_msgs: self.fn_output,
            buffer_default: Buffer {
                address: self.address,
                ..Default::default()
            },
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
