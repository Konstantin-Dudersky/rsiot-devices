use bitvec::{order::Msb0, view::BitView};

pub struct FifoEn {
    pub temp_fifo_en: bool,
    pub xg_fifo_en: bool,
    pub yg_fifo_en: bool,
    pub zg_fifo_en: bool,
    pub accel_fifo_en: bool,
    pub slv2_fifo_en: bool,
    pub slv1_fifo_en: bool,
    pub slv0_fifo_en: bool,
}

impl FifoEn {
    pub fn encode(&self) -> u8 {
        let mut byte: u8 = 0;
        let bits = byte.view_bits_mut::<Msb0>();

        bits.set(0, self.slv0_fifo_en);
        bits.set(1, self.slv1_fifo_en);
        bits.set(2, self.slv2_fifo_en);
        bits.set(3, self.accel_fifo_en);
        bits.set(4, self.zg_fifo_en);
        bits.set(5, self.yg_fifo_en);
        bits.set(6, self.xg_fifo_en);
        bits.set(7, self.temp_fifo_en);

        bits.reverse();

        byte
    }
}
