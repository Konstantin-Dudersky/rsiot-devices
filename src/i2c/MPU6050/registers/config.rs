use bitvec::{field::BitField, order::Msb0, view::BitView};

use super::digital_low_pass_filter::DigitalLowPassFilter;

pub struct Config {
    pub digital_low_pass_filter: DigitalLowPassFilter,
}

impl Config {
    pub fn encode(&self) -> u8 {
        let mut byte: u8 = 0;
        let bits = byte.view_bits_mut::<Msb0>();

        bits[5..8].store_be(self.digital_low_pass_filter as u8);

        byte
    }
}
