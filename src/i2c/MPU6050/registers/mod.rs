pub mod accel_config;
pub mod clock_source;
pub mod config;
pub mod digital_low_pass_filter;
pub mod dmp_firmware;
pub mod fifo_en;
pub mod gyro_config;
pub mod int_enable;
pub mod pwr_mgmt_1;
pub mod user_ctrl;

use bitvec::{field::BitField, order::Msb0, view::BitView};
use rsiot::components_config::i2c_master::Operation;

use crate::i2c::MPU6050::registers::{
    self,
    dmp_firmware::{BANK_SIZE, CHUNK_SIZE},
};

use super::{AccelFullScale, GyroFullScale};

pub const DMP_FIFO_LEN: usize = 28;

enum Register {
    // Accelerometer Calibration Registers
    /// High byte of X-axis accelerometer offset
    AccelOffsetXH = 0x06,

    /// Sample Rate Divider register (0x19)
    /// Sets the sample rate by dividing the gyroscope output
    SmpRtDiv = 0x19,

    /// Configuration register (0x1A)
    /// Controls the digital low pass filter and external sync
    Config = 0x1A,

    /// Gyroscope Configuration register (0x1B)
    /// Controls full-scale range
    GyroConfig = 0x1B,

    /// Accelerometer Configuration register (0x1C)
    /// Controls full-scale range and high pass filter
    AccelConfig = 0x1C,

    /// FIFO Enable register (0x23)
    /// Controls which sensor data goes to FIFO
    FifoEn = 0x23,

    /// Interrupt Enable register (0x38)
    /// Controls which interrupts are enabled
    IntEnable = 0x38,

    /// High byte of X-axis acceleration
    AccelXH = 0x3B,

    /// High byte of temperature reading
    TempOutH = 0x41,

    /// User Control register (0x6A)
    /// Controls FIFO and I2C configuration
    UserCtrl = 0x6A,

    /// Power Management 1 register (0x6B)
    /// Controls device power state, clock source, and reset
    PwrMgmt1 = 0x6B,

    // DMP Registers
    /// DMP Bank Select
    BankSel = 0x6D,
    /// DMP Memory Start Address
    MemStartAddr = 0x6E,
    /// DMP Memory Read Write
    MemRw = 0x6F,
    /// DMP Program Start Address
    PrgmStart = 0x70,

    /// High byte of FIFO byte count
    FifoCountH = 0x72,

    /// FIFO Read Write register
    FifoRw = 0x74,
}

// Operations --------------------------------------------------------------------------------------
pub struct Operations {}

impl Operations {
    pub fn read_calibration_offsets() -> Operation {
        Operation::WriteRead {
            write_data: vec![Register::AccelOffsetXH as u8],
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
        accel_bytes[0] = Register::AccelOffsetXH as u8; // Адрес регистра
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

    pub fn write_sample_rate_divider(value: u8) -> Operation {
        Operation::Write {
            write_data: vec![Register::SmpRtDiv as u8, value],
        }
    }

    pub fn write_config(value: config::Config) -> Operation {
        Operation::Write {
            write_data: vec![Register::Config as u8, value.encode()],
        }
    }

    pub fn write_gyro_config(value: gyro_config::GyroConfig) -> Operation {
        Operation::Write {
            write_data: vec![Register::GyroConfig as u8, value.encode()],
        }
    }

    pub fn write_accel_config(value: accel_config::AccelConfig) -> Operation {
        Operation::Write {
            write_data: vec![Register::AccelConfig as u8, value.encode()],
        }
    }

    pub fn write_fifo_en(value: fifo_en::FifoEn) -> Operation {
        Operation::Write {
            write_data: vec![Register::FifoEn as u8, value.encode()],
        }
    }

    pub fn write_int_enable(value: int_enable::IntEnable) -> Operation {
        Operation::Write {
            write_data: vec![Register::IntEnable as u8, value.encode()],
        }
    }

    pub fn read_accel_gyro() -> Operation {
        Operation::WriteRead {
            write_data: vec![Register::AccelXH as u8],
            read_size: 14,
        }
    }

    pub fn read_temperature() -> Operation {
        Operation::WriteRead {
            write_data: vec![Register::TempOutH as u8],
            read_size: 2,
        }
    }

    pub fn write_user_ctrl(value: user_ctrl::UserCtrl) -> Operation {
        Operation::Write {
            write_data: vec![Register::UserCtrl as u8, value.encode()],
        }
    }

    pub fn write_pwr_mgmt_1(value: pwr_mgmt_1::PwrMgmt1) -> Operation {
        Operation::Write {
            write_data: vec![Register::PwrMgmt1 as u8, value.encode()],
        }
    }

    pub fn write_dmp_firmware() -> Vec<Operation> {
        let mut ops = Vec::new();

        let data = registers::dmp_firmware::FIRMWARE;

        for (bank, chunk) in data.chunks(BANK_SIZE).enumerate() {
            let bank_ops = Self::write_dmp_firmware_bank(bank as u8, chunk);
            ops.extend(bank_ops);
        }

        ops
    }

    fn write_dmp_firmware_bank(bank: u8, data: &[u8]) -> Vec<Operation> {
        let mut ops = Vec::new();

        let op = Operation::Write {
            write_data: vec![Register::BankSel as u8, bank],
        };
        ops.push(op);

        for (i, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
            let mut prolog_and_chunk: [u8; CHUNK_SIZE + 1] = [0; CHUNK_SIZE + 1];
            prolog_and_chunk[0] = Register::MemRw as u8;
            for (i, b) in chunk.iter().enumerate() {
                prolog_and_chunk[i + 1] = *b;
            }

            let op = Operation::Write {
                write_data: vec![Register::MemStartAddr as u8, (i * CHUNK_SIZE) as u8],
            };
            ops.push(op);

            let op = Operation::Write {
                write_data: prolog_and_chunk.to_vec(),
            };
            ops.push(op);
        }

        ops
    }

    pub fn write_boot_firmware() -> Operation {
        Operation::Write {
            write_data: vec![Register::PrgmStart as u8, 0x04, 0x00],
        }
    }

    pub fn read_fifo_count() -> Operation {
        Operation::WriteRead {
            write_data: vec![Register::FifoCountH as u8],
            read_size: 2,
        }
    }

    pub fn read_fifo_rw(read_size: usize) -> Operation {
        Operation::WriteRead {
            write_data: vec![Register::FifoRw as u8],
            read_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_calibration_offsets() {
        let operation = Operations::write_calibration_offsets(-4130, 2939, 1430, 0, 0, 0);
        if let Operation::Write { write_data } = operation[0].clone() {
            assert_eq!(write_data, [0x06, 0xef, 0xde, 0x0b, 0x7b, 0x05, 0x96]);
        } else {
            panic!("Wrong operation")
        }
    }
}
