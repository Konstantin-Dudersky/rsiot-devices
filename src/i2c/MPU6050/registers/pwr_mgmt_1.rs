use bitvec::{field::BitField, order::Msb0, view::BitView};

use super::clock_source::ClockSource;

pub struct PwrMgmt1 {
    pub device_reset: bool,
    pub sleep: bool,
    pub cycle: bool,
    pub temp_dis: bool,
    pub clc_sel: ClockSource,
}
impl PwrMgmt1 {
    pub fn encode(&self) -> u8 {
        let mut byte: u8 = 0;
        let bits = byte.view_bits_mut::<Msb0>();

        bits.set(0, self.device_reset);
        bits.set(1, self.sleep);
        bits.set(2, self.cycle);
        bits.set(4, self.temp_dis);
        bits[5..8].store_be(self.clc_sel as u8);

        byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_pwr_mgmt_1() {
        let config = PwrMgmt1 {
            device_reset: true,
            sleep: false,
            cycle: true,
            temp_dis: false,
            clc_sel: super::ClockSource::External19200,
        };
        let encode = config.encode();

        assert_eq!(encode, 0b1010_0101);
    }
}
