use bitvec::{
    field::BitField,
    order::{Lsb0, Msb0},
    view::BitView,
};
use rsiot::components_config::i2c_master::Operation;

pub struct MPU6050Registers {}

impl MPU6050Registers {
    pub fn read_calibration_offsets() -> Operation {
        Operation::WriteRead {
            write_data: vec![0x06],
            read_size: 19,
        }
    }

    pub fn write_calibration_offsets(
        accel_x: i16,
        accel_y: i16,
        accel_z: i16,
        gyro_x: i16,
        gyro_y: i16,
        gyro_z: i16,
    ) -> Vec<Operation> {
        let mut accel_bytes = [0; 7];
        accel_bytes[0] = 0x06; // Адрес регистра
        let accel_bits = accel_bytes.view_bits_mut::<Msb0>();
        accel_bits[8..24].store_be(accel_x);
        accel_bits[24..40].store_be(accel_y);
        accel_bits[40..56].store_be(accel_z);

        let mut gyro_bytes = [0; 7];
        gyro_bytes[0] = 0x13; // Адрес регистра
        let gyro_bits = gyro_bytes.view_bits_mut::<Msb0>();
        gyro_bits[8..24].store_be(gyro_x);
        gyro_bits[24..40].store_be(gyro_y);
        gyro_bits[40..56].store_be(gyro_z);

        vec![
            Operation::Write {
                write_data: accel_bytes.to_vec(),
            },
            Operation::Write {
                write_data: gyro_bytes.to_vec(),
            },
        ]
    }

    pub fn read_config() -> Operation {
        Operation::WriteRead {
            write_data: vec![0x1B],
            read_size: 2,
        }
    }

    pub fn write_gyro_config(fs_sel: &FsSel) -> Operation {
        let mut byte: u8 = 0;
        let bits = byte.view_bits_mut::<Lsb0>();

        bits.set(4, matches!(fs_sel, FsSel::_1000DPS | FsSel::_2000DPS));
        bits.set(3, matches!(fs_sel, FsSel::_500DPS | FsSel::_2000DPS));

        Operation::Write {
            write_data: vec![0x1B, byte],
        }
    }

    pub fn write_accel_config(afs_sel: &AfsSel) -> Operation {
        let mut byte: u8 = 0;
        let bits = byte.view_bits_mut::<Lsb0>();

        bits.set(4, matches!(afs_sel, AfsSel::_8G | AfsSel::_16G));
        bits.set(3, matches!(afs_sel, AfsSel::_4G | AfsSel::_16G));

        Operation::Write {
            write_data: vec![0x1C, byte],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FsSel {
    #[default]
    _250DPS,
    _500DPS,
    _1000DPS,
    _2000DPS,
}

// Selects the full scale range of the accelerometer outputs
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AfsSel {
    #[default]
    _2G,
    _4G,
    _8G,
    _16G,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_calibration_offsets() {
        let operation = MPU6050Registers::write_calibration_offsets(-4130, 2939, 1430, 0, 0, 0);
        if let Operation::Write { write_data } = operation[0].clone() {
            assert_eq!(write_data, [0x06, 0xef, 0xde, 0x0b, 0x7b, 0x05, 0x96]);
        } else {
            panic!("Wrong operation")
        }
    }

    #[test]
    fn write_gyro_config() {
        let operation = MPU6050Registers::write_gyro_config(&FsSel::_250DPS);
        if let Operation::Write { write_data } = operation {
            assert_eq!(write_data, [0x1B, 0x00]);
        } else {
            panic!("Wrong operation")
        }

        let operation = MPU6050Registers::write_gyro_config(&FsSel::_500DPS);
        if let Operation::Write { write_data } = operation {
            assert_eq!(write_data, [0x1B, 0b0000_1000]);
        } else {
            panic!("Wrong operation")
        }

        let operation = MPU6050Registers::write_gyro_config(&FsSel::_1000DPS);
        if let Operation::Write { write_data } = operation {
            assert_eq!(write_data, [0x1B, 0b0001_0000]);
        } else {
            panic!("Wrong operation")
        }

        let operation = MPU6050Registers::write_gyro_config(&FsSel::_2000DPS);
        if let Operation::Write { write_data } = operation {
            assert_eq!(write_data, [0x1B, 0b001_1000]);
        } else {
            panic!("Wrong operation")
        }
    }

    #[test]
    fn write_accel_config() {
        let operation = MPU6050Registers::write_accel_config(&AfsSel::_2G);
        if let Operation::Write { write_data } = operation {
            assert_eq!(write_data, [0x1C, 0x00]);
        } else {
            panic!("Wrong operation")
        }

        let operation = MPU6050Registers::write_accel_config(&AfsSel::_4G);
        if let Operation::Write { write_data } = operation {
            assert_eq!(write_data, [0x1C, 0b0000_1000]);
        } else {
            panic!("Wrong operation")
        }

        let operation = MPU6050Registers::write_accel_config(&AfsSel::_8G);
        if let Operation::Write { write_data } = operation {
            assert_eq!(write_data, [0x1C, 0b0001_0000]);
        } else {
            panic!("Wrong operation")
        }

        let operation = MPU6050Registers::write_accel_config(&AfsSel::_16G);
        if let Operation::Write { write_data } = operation {
            assert_eq!(write_data, [0x1C, 0b001_1000]);
        } else {
            panic!("Wrong operation")
        }
    }
}
