use bitvec::{order::Msb0, view::BitView};

pub struct UserCtrl {
    /// 1 = активация DMP
    pub reserved_7: bool,
    pub fifo_en: bool,
    pub i2c_mst_en: bool,
    pub i2c_if_dis: bool,
    pub fifo_reset: bool,
    pub i2c_mst_reset: bool,
    pub sig_cond_reset: bool,
}

impl UserCtrl {
    pub fn encode(&self) -> u8 {
        let mut byte = 0u8;
        let bits = byte.view_bits_mut::<Msb0>();

        bits.set(0, self.reserved_7);
        bits.set(1, self.fifo_en);
        bits.set(2, self.i2c_mst_en);
        bits.set(3, self.i2c_if_dis);
        bits.set(5, self.fifo_reset);
        bits.set(6, self.i2c_mst_reset);
        bits.set(7, self.sig_cond_reset);

        byte
    }
}
