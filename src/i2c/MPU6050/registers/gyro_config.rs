use bitvec::{field::BitField, order::Msb0, view::BitView};

use super::GyroFullScale;

pub struct GyroConfig {
    pub xg_st: bool,
    pub yg_st: bool,
    pub zg_st: bool,
    pub full_scale: GyroFullScale,
}

impl GyroConfig {
    pub fn encode(&self) -> u8 {
        let mut byte: u8 = 0;
        let bits = byte.view_bits_mut::<Msb0>();

        bits.set(0, self.xg_st);
        bits.set(1, self.yg_st);
        bits.set(2, self.zg_st);
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
        let data = GyroConfig {
            xg_st: false,
            yg_st: false,
            zg_st: false,
            full_scale: GyroFullScale::Deg250,
        }
        .encode();
        assert_eq!(data, 0x00);

        let data = GyroConfig {
            xg_st: false,
            yg_st: false,
            zg_st: false,
            full_scale: GyroFullScale::Deg500,
        }
        .encode();
        assert_eq!(data, 0b0000_1000);

        let data = GyroConfig {
            xg_st: false,
            yg_st: false,
            zg_st: false,
            full_scale: GyroFullScale::Deg1000,
        }
        .encode();
        assert_eq!(data, 0b0001_0000);

        let data = GyroConfig {
            xg_st: false,
            yg_st: false,
            zg_st: false,
            full_scale: GyroFullScale::Deg2000,
        }
        .encode();
        assert_eq!(data, 0b001_1000);
    }
}
