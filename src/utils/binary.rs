use bytes::{Buf, BufMut, Bytes};
use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum BinaryError {
    #[error("Not enough bytes in buffer: needed {needed}, remaining {remaining}")]
    UnexpectedEof { needed: usize, remaining: usize },

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("VarInt/VarLong is too long (max bytes: {max_bytes})")]
    VarIntTooLong { max_bytes: usize },

    #[error("VarInt/VarLong value out of range for target type")]
    VarIntOutOfRange,

    #[error("Invalid UTF-8 string data: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),

    #[error("Invalid data encountered: {0}")]
    InvalidData(String),
}

pub type Result<T> = std::result::Result<T, BinaryError>;

const MAGIC_BYTES: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

macro_rules! check_remaining {
    ($buf:expr, $len:expr) => {
        let needed = $len;
        let remaining = $buf.remaining();
        if remaining < needed {
            return Err(BinaryError::UnexpectedEof { needed, remaining });
        }
    };
}

pub trait BinaryReader: Buf {
    #[inline]
    #[must_use]
    fn read_u8(&mut self) -> Result<u8> {
        check_remaining!(self, 1);
        Ok(self.get_u8())
    }

    #[inline]
    #[must_use]
    fn read_i8(&mut self) -> Result<i8> {
        check_remaining!(self, 1);
        Ok(self.get_i8())
    }

    #[inline]
    #[must_use]
    fn read_bool(&mut self) -> Result<bool> {
        check_remaining!(self, 1);
        Ok(self.get_u8() != 0)
    }

    #[inline]
    #[must_use]
    fn read_u16_le(&mut self) -> Result<u16> {
        check_remaining!(self, 2);
        Ok(self.get_u16_le())
    }

    #[inline]
    #[must_use]
    fn read_i16_le(&mut self) -> Result<i16> {
        check_remaining!(self, 2);
        Ok(self.get_i16_le())
    }

    #[inline]
    #[must_use]
    fn read_u24_le(&mut self) -> Result<u32> {
        check_remaining!(self, 3);
        Ok(self.get_uint_le(3) as u32)
    }

    #[inline]
    #[must_use]
    fn read_i24_le(&mut self) -> Result<i32> {
        check_remaining!(self, 3);
        let uval = self.get_uint_le(3);
        Ok(if (uval & 0x00800000) != 0 {
            (uval | 0xFF000000) as i32
        } else {
            uval as i32
        })
    }

    #[inline]
    #[must_use]
    fn read_u32_le(&mut self) -> Result<u32> {
        check_remaining!(self, 4);
        Ok(self.get_u32_le())
    }

    #[inline]
    #[must_use]
    fn read_i32_le(&mut self) -> Result<i32> {
        check_remaining!(self, 4);
        Ok(self.get_i32_le())
    }

    #[inline]
    #[must_use]
    fn read_u64_le(&mut self) -> Result<u64> {
        check_remaining!(self, 8);
        Ok(self.get_u64_le())
    }

    #[inline]
    #[must_use]
    fn read_i64_le(&mut self) -> Result<i64> {
        check_remaining!(self, 8);
        Ok(self.get_i64_le())
    }

    #[inline]
    #[must_use]
    fn read_u128_le(&mut self) -> Result<u128> {
        check_remaining!(self, 16);
        Ok(self.get_u128_le())
    }

    #[inline]
    #[must_use]
    fn read_i128_le(&mut self) -> Result<i128> {
        check_remaining!(self, 16);
        Ok(self.get_i128_le())
    }

    #[inline]
    #[must_use]
    fn read_u16_be(&mut self) -> Result<u16> {
        check_remaining!(self, 2);
        Ok(self.get_u16())
    }

    #[inline]
    #[must_use]
    fn read_i16_be(&mut self) -> Result<i16> {
        check_remaining!(self, 2);
        Ok(self.get_i16())
    }

    #[inline]
    #[must_use]
    fn read_u24_be(&mut self) -> Result<u32> {
        check_remaining!(self, 3);
        Ok(self.get_uint(3) as u32)
    }

    #[inline]
    #[must_use]
    fn read_i24_be(&mut self) -> Result<i32> {
        check_remaining!(self, 3);
        let unsigned_value = self.get_uint(3);
        Ok(if (unsigned_value & 0x00800000) != 0 {
            (unsigned_value | 0xFF000000) as i32
        } else {
            unsigned_value as i32
        })
    }

    #[inline]
    #[must_use]
    fn read_u32_be(&mut self) -> Result<u32> {
        check_remaining!(self, 4);
        Ok(self.get_u32())
    }

    #[inline]
    #[must_use]
    fn read_i32_be(&mut self) -> Result<i32> {
        check_remaining!(self, 4);
        Ok(self.get_i32())
    }

    #[inline]
    #[must_use]
    fn read_u64_be(&mut self) -> Result<u64> {
        check_remaining!(self, 8);
        Ok(self.get_u64())
    }

    #[inline]
    #[must_use]
    fn read_i64_be(&mut self) -> Result<i64> {
        check_remaining!(self, 8);
        Ok(self.get_i64())
    }

    #[inline]
    #[must_use]
    fn read_u128_be(&mut self) -> Result<u128> {
        check_remaining!(self, 16);
        Ok(self.get_u128())
    }

    #[inline]
    #[must_use]
    fn read_i128_be(&mut self) -> Result<i128> {
        check_remaining!(self, 16);
        Ok(self.get_i128())
    }

    #[inline]
    #[must_use]
    fn read_f32_le(&mut self) -> Result<f32> {
        check_remaining!(self, 4);
        Ok(self.get_f32_le())
    }

    #[inline]
    #[must_use]
    fn read_f64_le(&mut self) -> Result<f64> {
        check_remaining!(self, 8);
        Ok(self.get_f64_le())
    }

    #[inline]
    #[must_use]
    fn read_f32_be(&mut self) -> Result<f32> {
        check_remaining!(self, 4);
        Ok(self.get_f32())
    }

    #[inline]
    #[must_use]
    fn read_f64_be(&mut self) -> Result<f64> {
        check_remaining!(self, 8);
        Ok(self.get_f64())
    }

    #[must_use]
    fn read_varu32(&mut self) -> Result<u32> {
        let mut value: u32 = 0;
        let mut shift: u32 = 0;
        const MAX_BYTES: usize = 5;

        for i in 0..MAX_BYTES {
            check_remaining!(self, 1);
            let byte = self.get_u8();

            value |= ((byte & 0x7F) as u32) << shift;

            if byte & 0x80 == 0 {
                if i == MAX_BYTES - 1 && (byte >> 4) != 0 {
                    return Err(BinaryError::VarIntOutOfRange);
                }
                return Ok(value);
            }

            shift += 7;
            if shift > 28 {}
        }

        Err(BinaryError::VarIntTooLong {
            max_bytes: MAX_BYTES,
        })
    }

    #[inline]
    #[must_use]
    fn read_vari32(&mut self) -> Result<i32> {
        let unsigned = self.read_varu32()?;
        Ok((unsigned >> 1) as i32 ^ -((unsigned & 1) as i32))
    }

    #[must_use]
    fn read_varu64(&mut self) -> Result<u64> {
        let mut value: u64 = 0;
        let mut shift: u32 = 0;
        const MAX_BYTES: usize = 10;

        for i in 0..MAX_BYTES {
            check_remaining!(self, 1);
            let byte = self.get_u8();

            value |= ((byte & 0x7F) as u64) << shift;

            if byte & 0x80 == 0 {
                if i == MAX_BYTES - 1 && (byte >> 1) != 0 {
                    return Err(BinaryError::VarIntOutOfRange);
                }
                return Ok(value);
            }

            shift += 7;
            if shift > 63 {
                return Err(BinaryError::VarIntOutOfRange);
            }
        }

        Err(BinaryError::VarIntTooLong {
            max_bytes: MAX_BYTES,
        })
    }

    #[inline]
    #[must_use]
    fn read_vari64(&mut self) -> Result<i64> {
        let unsigned = self.read_varu64()?;
        Ok((unsigned >> 1) as i64 ^ -((unsigned & 1) as i64))
    }

    #[must_use]
    fn read_bytes_varint_len(&mut self) -> Result<Bytes> {
        let len = self.read_varu32()? as usize;
        check_remaining!(self, len);
        Ok(self.copy_to_bytes(len))
    }

    #[must_use]
    fn read_string_varint_len(&mut self) -> Result<String> {
        let bytes = self.read_bytes_varint_len()?;
        String::from_utf8(bytes.to_vec()).map_err(BinaryError::from)
    }

    #[must_use]
    fn read_bytes(&mut self, len: usize) -> Result<Bytes> {
        check_remaining!(self, len);
        Ok(self.copy_to_bytes(len))
    }

    #[must_use]
    fn read_remaining_bytes(&mut self) -> Bytes {
        self.copy_to_bytes(self.remaining())
    }

    #[must_use]
    fn read_uuid_le(&mut self) -> Result<Uuid> {
        check_remaining!(self, 16);
        let lsb = self.get_u64_le();
        let msb = self.get_u64_le();
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&lsb.to_le_bytes());
        bytes[8..16].copy_from_slice(&msb.to_le_bytes());
        Ok(Uuid::from_bytes_le(bytes))
    }

    #[must_use]
    fn read_uuid_be(&mut self) -> Result<Uuid> {
        check_remaining!(self, 16);
        let mut bytes = [0u8; 16];
        self.copy_to_slice(&mut bytes);
        Ok(Uuid::from_bytes(bytes))
    }

    fn read_magic(&mut self) -> Result<bool> {
        const MAGIC_LEN: usize = MAGIC_BYTES.len();
        check_remaining!(self, MAGIC_LEN);
        let mut read_buffer = [0u8; MAGIC_LEN];
        self.copy_to_slice(&mut read_buffer);
        Ok(read_buffer == MAGIC_BYTES)
    }
}

impl<T: Buf> BinaryReader for T {}

pub trait BinaryWriter: BufMut {
    #[inline]
    fn write_u8(&mut self, value: u8) -> Result<()> {
        self.put_u8(value);
        Ok(())
    }

    #[inline]
    fn write_i8(&mut self, value: i8) -> Result<()> {
        self.put_i8(value);
        Ok(())
    }

    #[inline]
    fn write_bool(&mut self, value: bool) -> Result<()> {
        self.put_u8(if value { 1 } else { 0 });
        Ok(())
    }

    #[inline]
    fn write_u16_le(&mut self, value: u16) -> Result<()> {
        self.put_u16_le(value);
        Ok(())
    }

    #[inline]
    fn write_i16_le(&mut self, value: i16) -> Result<()> {
        self.put_i16_le(value);
        Ok(())
    }

    #[inline]
    fn write_u24_le(&mut self, value: u32) -> Result<()> {
        self.put_uint_le(value as u64, 3);
        Ok(())
    }

    #[inline]
    fn write_i24_le(&mut self, value: i32) -> Result<()> {
        self.put_uint_le(value as u64, 3);
        Ok(())
    }

    #[inline]
    fn write_u32_le(&mut self, value: u32) -> Result<()> {
        self.put_u32_le(value);
        Ok(())
    }

    #[inline]
    fn write_i32_le(&mut self, value: i32) -> Result<()> {
        self.put_i32_le(value);
        Ok(())
    }

    #[inline]
    fn write_u64_le(&mut self, value: u64) -> Result<()> {
        self.put_u64_le(value);
        Ok(())
    }

    #[inline]
    fn write_i64_le(&mut self, value: i64) -> Result<()> {
        self.put_i64_le(value);
        Ok(())
    }

    #[inline]
    fn write_u128_le(&mut self, value: u128) -> Result<()> {
        self.put_u128_le(value);
        Ok(())
    }

    #[inline]
    fn write_i128_le(&mut self, value: i128) -> Result<()> {
        self.put_i128_le(value);
        Ok(())
    }

    #[inline]
    fn write_u16_be(&mut self, value: u16) -> Result<()> {
        self.put_u16(value);
        Ok(())
    }

    #[inline]
    fn write_i16_be(&mut self, value: i16) -> Result<()> {
        self.put_i16(value);
        Ok(())
    }

    #[inline]
    fn write_u24_be(&mut self, value: u32) -> Result<()> {
        self.put_uint(value as u64, 3);
        Ok(())
    }

    #[inline]
    fn write_i24_be(&mut self, value: i32) -> Result<()> {
        self.put_uint(value as u64, 3);
        Ok(())
    }

    #[inline]
    fn write_u32_be(&mut self, value: u32) -> Result<()> {
        self.put_u32(value);
        Ok(())
    }

    #[inline]
    fn write_i32_be(&mut self, value: i32) -> Result<()> {
        self.put_i32(value);
        Ok(())
    }

    #[inline]
    fn write_u64_be(&mut self, value: u64) -> Result<()> {
        self.put_u64(value);
        Ok(())
    }

    #[inline]
    fn write_i64_be(&mut self, value: i64) -> Result<()> {
        self.put_i64(value);
        Ok(())
    }

    #[inline]
    fn write_u128_be(&mut self, value: u128) -> Result<()> {
        self.put_u128(value);
        Ok(())
    }

    #[inline]
    fn write_i128_be(&mut self, value: i128) -> Result<()> {
        self.put_i128(value);
        Ok(())
    }

    #[inline]
    fn write_f32_le(&mut self, value: f32) -> Result<()> {
        self.put_f32_le(value);
        Ok(())
    }

    #[inline]
    fn write_f64_le(&mut self, value: f64) -> Result<()> {
        self.put_f64_le(value);
        Ok(())
    }

    #[inline]
    fn write_f32_be(&mut self, value: f32) -> Result<()> {
        self.put_f32(value);
        Ok(())
    }

    #[inline]
    fn write_f64_be(&mut self, value: f64) -> Result<()> {
        self.put_f64(value);
        Ok(())
    }

    fn write_varu32(&mut self, mut value: u32) -> Result<()> {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.put_u8(byte);
            if value == 0 {
                return Ok(());
            }
        }
    }

    #[inline]
    fn write_vari32(&mut self, value: i32) -> Result<()> {
        let unsigned = (value << 1) ^ (value >> 31);
        self.write_varu32(unsigned as u32)
    }

    fn write_varu64(&mut self, mut value: u64) -> Result<()> {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.put_u8(byte);
            if value == 0 {
                return Ok(());
            }
        }
    }

    #[inline]
    fn write_vari64(&mut self, value: i64) -> Result<()> {
        let unsigned = (value << 1) ^ (value >> 63);
        self.write_varu64(unsigned as u64)
    }

    fn write_bytes_varint_len(&mut self, bytes: &[u8]) -> Result<()> {
        let len = u32::try_from(bytes.len()).map_err(|_| {
            BinaryError::InvalidData("Byte slice length exceeds u32::MAX".to_string())
        })?;
        self.write_varu32(len)?;
        self.put_slice(bytes);
        Ok(())
    }

    fn write_string_varint_len(&mut self, string: &str) -> Result<()> {
        self.write_bytes_varint_len(string.as_bytes())
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.put_slice(bytes);
        Ok(())
    }

    fn write_uuid_le(&mut self, uuid: &Uuid) -> Result<()> {
        self.put_slice(uuid.to_bytes_le().as_slice());
        Ok(())
    }

    fn write_uuid_be(&mut self, uuid: &Uuid) -> Result<()> {
        self.put_slice(uuid.as_bytes());
        Ok(())
    }

    fn write_magic(&mut self) -> Result<()> {
        self.put_slice(&MAGIC_BYTES);
        Ok(())
    }
}

impl<T: BufMut> BinaryWriter for T {}
