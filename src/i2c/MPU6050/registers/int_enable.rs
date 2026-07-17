use bitvec::{order::Msb0, view::BitView};

pub struct IntEnable {
    pub fifo_oflow_en: bool,
    pub i2c_mst_int_en: bool,
    pub data_rdy_en: bool,
}

impl IntEnable {
    pub fn encode(&self) -> u8 {
        let mut byte: u8 = 0;
        let bits = byte.view_bits_mut::<Msb0>();

        bits.set(0, self.data_rdy_en);
        bits.set(3, self.i2c_mst_int_en);
        bits.set(4, self.fifo_oflow_en);

        bits.reverse();

        byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let data = IntEnable {
            fifo_oflow_en: true,
            i2c_mst_int_en: false,
            data_rdy_en: true,
        }
        .encode();

        assert_eq!(data, 0b0001_0001);
    }
}
