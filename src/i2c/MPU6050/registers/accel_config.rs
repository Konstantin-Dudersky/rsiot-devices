use bitvec::{field::BitField, order::Msb0, view::BitView};

use super::AccelFullScale;

pub struct AccelConfig {
    pub xa_st: bool,
    pub ya_st: bool,
    pub za_st: bool,
    pub full_scale: AccelFullScale,
}

impl AccelConfig {
    pub fn encode(&self) -> u8 {
        let mut byte: u8 = 0;
        let bits = byte.view_bits_mut::<Msb0>();

        bits.set(0, self.xa_st);
        bits.set(1, self.ya_st);
        bits.set(2, self.za_st);
        bits[3..5].store_be(self.full_scale as u8);

        byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn write_accel_config() {
        let data = AccelConfig {
            xa_st: false,
            ya_st: false,
            za_st: false,
            full_scale: AccelFullScale::G2,
        }
        .encode();
        assert_eq!(data, 0x00);

        let data = AccelConfig {
            xa_st: false,
            ya_st: false,
            za_st: false,
            full_scale: AccelFullScale::G4,
        }
        .encode();
        assert_eq!(data, 0b0000_1000);

        let data = AccelConfig {
            xa_st: false,
            ya_st: false,
            za_st: false,
            full_scale: AccelFullScale::G8,
        }
        .encode();
        assert_eq!(data, 0b0001_0000);

        let data = AccelConfig {
            xa_st: false,
            ya_st: false,
            za_st: false,
            full_scale: AccelFullScale::G16,
        }
        .encode();
        assert_eq!(data, 0b001_1000);
    }
}
