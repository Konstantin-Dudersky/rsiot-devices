use std::time::Duration;

use async_trait::async_trait;
use bitvec::{field::BitField, order::Msb0, view::BitView};
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
    super::device_id,
    buffer::{Config, WriteData},
    request_kind::RequestKind,
    Buffer, DEVICE_NAME,
};

/// Датчик температуры и влажности AHT10
#[derive(Clone, Debug)]
pub struct Device<TMsg>
where
    TMsg: MsgDataBound,
{
    /// Адрес - 0x68
    pub address: I2cAddress,

    /// Период чтения данных
    pub request_period: Duration,

    /// Функция для записи времени
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
        let device_id = device_id(DEVICE_NAME, self.address);

        let periodic_requests = vec![ConfigPeriodicRequest {
            period: self.request_period,
            fn_requests: |buffer: &Buffer| {
                let mut requests = vec![];

                let req = FieldbusRequest::new(
                    buffer.config.address,
                    RequestKind::ReadValues,
                    vec![Operation::WriteRead {
                        write_data: vec![0x00],
                        read_size: 19,
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
                device_id,
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

    let req = FieldbusRequest::new(buffer.config.address, RequestKind::Init, vec![]);
    requests.push(req);

    requests
}

fn fn_buffer_to_request(buffer: &Buffer) -> anyhow::Result<Vec<FieldbusRequest>> {
    let mut requests = vec![];

    if buffer.write_data.need_write {
        let second = to_bcd(buffer.write_data.datetime.second);
        let minute = to_bcd(buffer.write_data.datetime.minute);
        let hour = to_bcd(buffer.write_data.datetime.hour);
        let day = to_bcd(buffer.write_data.datetime.day);
        let month = to_bcd(buffer.write_data.datetime.month);
        let year = to_bcd(buffer.write_data.datetime.year);

        let req = FieldbusRequest::new(
            buffer.config.address,
            RequestKind::WriteTime,
            vec![Operation::Write {
                write_data: vec![0x00, second, minute, hour, 1, day, month, year],
            }],
        );
        requests.push(req);
    }

    Ok(requests)
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
            warn!("Error reading: {}", err);
            return ResponseResult::error(err);
        }
    };

    match request_kind {
        RequestKind::Init => ResponseResult::ok_init_completed(),

        RequestKind::ReadValues => {
            let payload = &payload[0];

            buffer.read_data.datetime.second = from_bcd(payload[0x00]);
            buffer.read_data.datetime.minute = from_bcd(payload[0x01]);
            buffer.read_data.datetime.hour = from_bcd(payload[0x02]);
            buffer.read_data.datetime.day = from_bcd(payload[0x04]);
            buffer.read_data.datetime.month = from_bcd(payload[0x05]);
            buffer.read_data.datetime.year = from_bcd(payload[0x06]);
            buffer.read_data.temperature = decode_temperature(&[payload[0x11], payload[0x12]]);

            ResponseResult::ok()
        }

        RequestKind::WriteTime => {
            buffer.write_data.need_write = false;

            ResponseResult::ok()
        }
    }
}

pub fn to_bcd(value: u8) -> u8 {
    let mut data: u8 = 0;
    let bits = data.view_bits_mut::<Msb0>();

    let first_digit = value / 10;
    let second_digit = value - first_digit * 10;

    bits[0..4].store_be(first_digit);
    bits[4..8].store_be(second_digit);

    data
}

pub fn from_bcd(data: u8) -> u8 {
    let bits = data.view_bits::<Msb0>();

    let first_digit = bits[0..4].load_be::<u8>();
    let second_digit = bits[4..8].load_be::<u8>();

    first_digit * 10 + second_digit
}

pub fn decode_temperature(payload: &[u8; 2]) -> f32 {
    let temp_integer: u8 = payload[0];

    let temp_fraction: u8 = payload[1].view_bits::<Msb0>()[0..2].load_be::<u8>();
    let temp_fraction: f32 = temp_fraction as f32 * 0.25;

    temp_integer as f32 + temp_fraction
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_temperature() {
        assert_eq!(decode_temperature(&[0b0001_1001, 0b0100_0000]), 25.25);
    }

    #[test]
    fn test_to_bcd() {
        assert_eq!(to_bcd(99), 0b1001_1001);
        assert_eq!(to_bcd(00), 0b0000_0000);
        assert_eq!(to_bcd(73), 0b111_0011);
    }
}
