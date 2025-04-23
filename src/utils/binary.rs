//! # Amethyst Binary Utilities
//!
//! Provides traits and implementations for reading and writing binary data
//! efficiently and safely, tailored for Minecraft Bedrock Edition protocols.
//!
//! Uses the `bytes` crate for buffer manipulation.
//!
//! I know, bytes already have all of that but makes it easier for us to maintain it.
//!
use bytes::{Buf, BufMut, Bytes};
use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;
use uuid::Uuid;

// --- Error Handling ---

/// Errors that can occur during binary operations.
#[derive(Error, Debug)]
pub enum BinaryError {
    /// Not enough bytes remaining in the buffer for the requested operation.
    #[error("Not enough bytes in buffer: needed {needed}, remaining {remaining}")]
    UnexpectedEof { needed: usize, remaining: usize },

    /// An underlying I/O error occurred (though less common with `bytes`).
    #[error("I/O error: {0}")]
    Io(#[from] io::Error), // Should ideally not happen with Bytes/BytesMut unless custom Buf/BufMut impls are used

    /// VarInt or VarLong exceeded the maximum allowed bytes.
    #[error("VarInt/VarLong is too long (max bytes: {max_bytes})")]
    VarIntTooLong { max_bytes: usize },

    /// VarInt or VarLong value exceeds the capacity of the target type.
    #[error("VarInt/VarLong value out of range for target type")]
    VarIntOutOfRange, // Although the read might succeed, the conversion could fail

    /// Attempted to read a string that was not valid UTF-8.
    #[error("Invalid UTF-8 string data: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),

    /// An invalid value was encountered (e.g., invalid boolean byte).
    #[error("Invalid data encountered: {0}")]
    InvalidData(String),
}

/// Result type alias for binary operations.
pub type Result<T> = std::result::Result<T, BinaryError>;

// --- Helper Macro for Reading ---

/// Checks if the buffer has enough remaining bytes and returns an error if not.
macro_rules! check_remaining {
    ($buf:expr, $len:expr) => {
        let needed = $len;
        let remaining = $buf.remaining();
        if remaining < needed {
            return Err(BinaryError::UnexpectedEof { needed, remaining });
        }
    };
}

// --- Reading Extension Trait ---

/// Extension trait for `bytes::Buf` providing methods to read various data types.
pub trait BinaryReader: Buf {
    // --- Single Bytes ---

    /// Reads a single `u8` byte.
    #[inline]
    #[must_use]
    fn read_u8(&mut self) -> Result<u8> {
        check_remaining!(self, 1);
        Ok(self.get_u8())
    }

    /// Reads a single `i8` byte.
    #[inline]
    #[must_use]
    fn read_i8(&mut self) -> Result<i8> {
        check_remaining!(self, 1);
        Ok(self.get_i8())
    }

    /// Reads a single byte as a boolean (`0x00` is false, anything else is true).
    #[inline]
    #[must_use]
    fn read_bool(&mut self) -> Result<bool> {
        check_remaining!(self, 1);
        Ok(self.get_u8() != 0)
    }

    // --- Multi-byte Integers (Little Endian) ---

    /// Reads a `u16` in little-endian format.
    #[inline]
    #[must_use]
    fn read_u16_le(&mut self) -> Result<u16> {
        check_remaining!(self, 2);
        Ok(self.get_u16_le())
    }

    /// Reads an `i16` in little-endian format.
    #[inline]
    #[must_use]
    fn read_i16_le(&mut self) -> Result<i16> {
        check_remaining!(self, 2);
        Ok(self.get_i16_le())
    }

    /// Reads a `u32` representing a little-endian 24-bit unsigned integer (triad).
    #[inline]
    #[must_use]
    fn read_u24_le(&mut self) -> Result<u32> {
        check_remaining!(self, 3);
        Ok(self.get_uint_le(3) as u32) // Read 3 bytes as LE unsigned int
    }

    /// Reads an `i32` representing a little-endian 24-bit signed integer (triad).
    #[inline]
    #[must_use]
    fn read_i24_le(&mut self) -> Result<i32> {
        check_remaining!(self, 3);
        let uval = self.get_uint_le(3);
        // Sign extend if the highest bit (bit 23) is set
        Ok(if (uval & 0x00800000) != 0 {
            (uval | 0xFF000000) as i32 // Cast to i32 after setting higher bits
        } else {
            uval as i32
        })
    }

    /// Reads a `u32` in little-endian format.
    #[inline]
    #[must_use]
    fn read_u32_le(&mut self) -> Result<u32> {
        check_remaining!(self, 4);
        Ok(self.get_u32_le())
    }

    /// Reads an `i32` in little-endian format.
    #[inline]
    #[must_use]
    fn read_i32_le(&mut self) -> Result<i32> {
        check_remaining!(self, 4);
        Ok(self.get_i32_le())
    }

    /// Reads a `u64` in little-endian format.
    #[inline]
    #[must_use]
    fn read_u64_le(&mut self) -> Result<u64> {
        check_remaining!(self, 8);
        Ok(self.get_u64_le())
    }

    /// Reads an `i64` in little-endian format.
    #[inline]
    #[must_use]
    fn read_i64_le(&mut self) -> Result<i64> {
        check_remaining!(self, 8);
        Ok(self.get_i64_le())
    }

    /// Reads a `u128` in little-endian format.
    #[inline]
    #[must_use]
    fn read_u128_le(&mut self) -> Result<u128> {
        check_remaining!(self, 16);
        Ok(self.get_u128_le())
    }

    /// Reads an `i128` in little-endian format.
    #[inline]
    #[must_use]
    fn read_i128_le(&mut self) -> Result<i128> {
        check_remaining!(self, 16);
        Ok(self.get_i128_le())
    }

    // --- Multi-byte Integers (Big Endian) ---

    /// Reads a `u16` in big-endian format.
    #[inline]
    #[must_use]
    fn read_u16_be(&mut self) -> Result<u16> {
        check_remaining!(self, 2);
        Ok(self.get_u16())
    }

    /// Reads an `i16` in big-endian format.
    #[inline]
    #[must_use]
    fn read_i16_be(&mut self) -> Result<i16> {
        check_remaining!(self, 2);
        Ok(self.get_i16())
    }

    /// Reads a `u32` representing a big-endian 24-bit unsigned integer (triad).
    #[inline]
    #[must_use]
    fn read_u24_be(&mut self) -> Result<u32> {
        check_remaining!(self, 3);
        Ok(self.get_uint(3) as u32) // Read 3 bytes as BE unsigned int
    }

    /// Reads an `i32` representing a big-endian 24-bit signed integer (triad).
    #[inline]
    #[must_use]
    fn read_i24_be(&mut self) -> Result<i32> {
        check_remaining!(self, 3);
        let uval = self.get_uint(3);
        // Sign extend if the highest bit (bit 23) is set
        Ok(if (uval & 0x00800000) != 0 {
            (uval | 0xFF000000) as i32 // Cast to i32 after setting higher bits
        } else {
            uval as i32
        })
    }

    /// Reads a `u32` in big-endian format.
    #[inline]
    #[must_use]
    fn read_u32_be(&mut self) -> Result<u32> {
        check_remaining!(self, 4);
        Ok(self.get_u32())
    }

    /// Reads an `i32` in big-endian format.
    #[inline]
    #[must_use]
    fn read_i32_be(&mut self) -> Result<i32> {
        check_remaining!(self, 4);
        Ok(self.get_i32())
    }

    /// Reads a `u64` in big-endian format.
    #[inline]
    #[must_use]
    fn read_u64_be(&mut self) -> Result<u64> {
        check_remaining!(self, 8);
        Ok(self.get_u64())
    }

    /// Reads an `i64` in big-endian format.
    #[inline]
    #[must_use]
    fn read_i64_be(&mut self) -> Result<i64> {
        check_remaining!(self, 8);
        Ok(self.get_i64())
    }

    /// Reads a `u128` in big-endian format.
    #[inline]
    #[must_use]
    fn read_u128_be(&mut self) -> Result<u128> {
        check_remaining!(self, 16);
        Ok(self.get_u128())
    }

    /// Reads an `i128` in big-endian format.
    #[inline]
    #[must_use]
    fn read_i128_be(&mut self) -> Result<i128> {
        check_remaining!(self, 16);
        Ok(self.get_i128())
    }

    // --- Floating Point (Little Endian) ---

    /// Reads an `f32` in little-endian format.
    #[inline]
    #[must_use]
    fn read_f32_le(&mut self) -> Result<f32> {
        check_remaining!(self, 4);
        Ok(self.get_f32_le())
    }

    /// Reads an `f64` in little-endian format.
    #[inline]
    #[must_use]
    fn read_f64_le(&mut self) -> Result<f64> {
        check_remaining!(self, 8);
        Ok(self.get_f64_le())
    }

    // --- Floating Point (Big Endian) ---

    /// Reads an `f32` in big-endian format.
    #[inline]
    #[must_use]
    fn read_f32_be(&mut self) -> Result<f32> {
        check_remaining!(self, 4);
        Ok(self.get_f32())
    }

    /// Reads an `f64` in big-endian format.
    #[inline]
    #[must_use]
    fn read_f64_be(&mut self) -> Result<f64> {
        check_remaining!(self, 8);
        Ok(self.get_f64())
    }

    // --- Variable Length Integers ---

    /// Reads an unsigned 32-bit variable-length integer (VarInt).
    #[must_use]
    fn read_varu32(&mut self) -> Result<u32> {
        let mut value: u32 = 0;
        let mut shift: u32 = 0;
        const MAX_BYTES: usize = 5; // Max bytes for a 32-bit VarInt

        for i in 0..MAX_BYTES {
            check_remaining!(self, 1);
            let byte = self.get_u8();
            // Check remaining *before* getting the byte to avoid advancing on error

            value |= ((byte & 0x7F) as u32) << shift;

            if byte & 0x80 == 0 {
                // Check if the value could have overflowed (only possible if last byte had high bits set unnecessarily)
                // A minimal encoding requires that the last byte does not have bits set beyond the 7th,
                // unless it's the 5th byte and needed for the highest bits of the u32.
                // This check prevents overly long encodings.
                // For u32, the 5th byte can only have the lower 4 bits set (32 = 5 * 7 - 3).
                if i == MAX_BYTES - 1 && (byte >> 4) != 0 {
                    return Err(BinaryError::VarIntOutOfRange);
                }
                return Ok(value);
            }

            shift += 7;
            // Check potential overflow before next iteration's shift
            // If shift will become >= 32, the next byte must be the last one and valid.
            if shift > 28 { // Max shift for u32 within 5 bytes is 28
                // If we need more bits than 32, it's an error unless it's exactly the 5th byte
                // This condition is implicitly handled by the loop range and the check above.
                // If VarIntOutOfRange occurred, we already returned Err.
            }
        }

        Err(BinaryError::VarIntTooLong { max_bytes: MAX_BYTES })
    }

    /// Reads a signed 32-bit variable-length integer (VarInt), using ZigZag encoding.
    #[inline]
    #[must_use]
    fn read_vari32(&mut self) -> Result<i32> {
        let unsigned = self.read_varu32()?;
        // ZigZag decode: (n >> 1) ^ -(n & 1)
        Ok((unsigned >> 1) as i32 ^ -((unsigned & 1) as i32))
    }

    /// Reads an unsigned 64-bit variable-length integer (VarLong).
    #[must_use]
    fn read_varu64(&mut self) -> Result<u64> {
        let mut value: u64 = 0;
        let mut shift: u32 = 0;
        const MAX_BYTES: usize = 10; // Max bytes for a 64-bit VarLong

        for i in 0..MAX_BYTES {
            check_remaining!(self, 1);
            let byte = self.get_u8();

            value |= ((byte & 0x7F) as u64) << shift;

            if byte & 0x80 == 0 {
                // Check for non-minimal encoding on the last byte for u64
                // The 10th byte can only use 1 bit (64 = 10 * 7 - 6)
                if i == MAX_BYTES - 1 && (byte >> 1) != 0 {
                    return Err(BinaryError::VarIntOutOfRange);
                }
                return Ok(value);
            }

            shift += 7;
            if shift > 63 { // Max shift for u64 within 10 bytes is 63
                // This implies we have more bits than fit in a u64
                return Err(BinaryError::VarIntOutOfRange);
            }
        }

        Err(BinaryError::VarIntTooLong { max_bytes: MAX_BYTES })
    }

    /// Reads a signed 64-bit variable-length integer (VarLong), using ZigZag encoding.
    #[inline]
    #[must_use]
    fn read_vari64(&mut self) -> Result<i64> {
        let unsigned = self.read_varu64()?;
        // ZigZag decode: (n >> 1) ^ -(n & 1)
        Ok((unsigned >> 1) as i64 ^ -((unsigned & 1) as i64))
    }

    // --- Slices and Strings ---

    /// Reads a byte slice prefixed with a `VarInt` length.
    /// Returns a `Bytes` slice which avoids copying when possible.
    #[must_use]
    fn read_bytes_varint_len(&mut self) -> Result<Bytes> {
        let len = self.read_varu32()? as usize;
        check_remaining!(self, len);
        Ok(self.copy_to_bytes(len))
    }

    /// Reads a UTF-8 string prefixed with a `VarInt` length.
    #[must_use]
    fn read_string_varint_len(&mut self) -> Result<String> {
        let bytes = self.read_bytes_varint_len()?;
        String::from_utf8(bytes.to_vec()).map_err(BinaryError::from)
        // Convert to Vec because from_utf8 takes ownership or requires a &Vec
        // This involves a copy if Bytes wasn't contiguous.
    }

    /// Reads a byte slice with a fixed length.
    /// Returns a `Bytes` slice which avoids copying when possible.
    #[must_use]
    fn read_bytes(&mut self, len: usize) -> Result<Bytes> {
        check_remaining!(self, len);
        Ok(self.copy_to_bytes(len))
    }

    /// Reads the remaining bytes in the buffer.
    #[must_use]
    fn read_remaining_bytes(&mut self) -> Bytes {
        self.copy_to_bytes(self.remaining())
    }

    // --- Complex Types ---

    /// Reads a `Uuid` (16 bytes) in little-endian format (standard for MCBE).
    #[must_use]
    fn read_uuid_le(&mut self) -> Result<Uuid> {
        check_remaining!(self, 16);
        // UUIDs are often represented as two u64s or directly as bytes.
        // The `uuid` crate expects bytes in network order (big-endian fields),
        // but MCBE often uses little-endian *storage* for the whole 16 bytes.
        // We'll read LE u64s and reconstruct, assuming standard MCBE practice.
        let lsb = self.get_u64_le(); // Least significant bits first in buffer
        let msb = self.get_u64_le(); // Most significant bits second in buffer
        // Construct Uuid from msb and lsb according to RFC 4122 internal field order.
        // This requires byte shuffling if we read LE u64s. Easier to read bytes:
        // check_remaining!(self, 16);
        // let mut bytes = [0u8; 16];
        // self.copy_to_slice(&mut bytes);
        // Ok(Uuid::from_bytes_le(bytes)) // Use uuid crate's LE constructor
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&lsb.to_le_bytes());
        bytes[8..16].copy_from_slice(&msb.to_le_bytes());
        Ok(Uuid::from_bytes_le(bytes)) // Correct way with uuid crate v1.6+

        // If direct u64 reading was strictly required by spec AND needed BE fields:
        // Ok(Uuid::from_u64_pair(msb.swap_bytes(), lsb.swap_bytes())) // Example IF fields were BE
    }

    /// Reads a `Uuid` (16 bytes) in big-endian format.
    #[must_use]
    fn read_uuid_be(&mut self) -> Result<Uuid> {
        check_remaining!(self, 16);
        let mut bytes = [0u8; 16];
        self.copy_to_slice(&mut bytes); // Read bytes directly in their BE order
        Ok(Uuid::from_bytes(bytes)) // Default Uuid constructor expects BE bytes
    }
}

// Implement the trait for all types that implement `Buf`.
impl<T: Buf> BinaryReader for T {}

// --- Writing Extension Trait ---

/// Extension trait for `bytes::BufMut` providing methods to write various data types.
pub trait BinaryWritter: BufMut {
    // --- Single Bytes ---

    /// Writes a single `u8` byte.
    #[inline]
    fn write_u8(&mut self, value: u8) -> Result<()> {
        self.put_u8(value);
        Ok(())
    }

    /// Writes a single `i8` byte.
    #[inline]
    fn write_i8(&mut self, value: i8) -> Result<()> {
        self.put_i8(value);
        Ok(())
    }

    /// Writes a boolean as a single byte (`0x01` for true, `0x00` for false).
    #[inline]
    fn write_bool(&mut self, value: bool) -> Result<()> {
        self.put_u8(if value { 1 } else { 0 });
        Ok(())
    }

    // --- Multi-byte Integers (Little Endian) ---

    /// Writes a `u16` in little-endian format.
    #[inline]
    fn write_u16_le(&mut self, value: u16) -> Result<()> {
        self.put_u16_le(value);
        Ok(())
    }

    /// Writes an `i16` in little-endian format.
    #[inline]
    fn write_i16_le(&mut self, value: i16) -> Result<()> {
        self.put_i16_le(value);
        Ok(())
    }

    /// Writes a `u32` as a little-endian 24-bit unsigned integer (triad).
    /// Value must be <= 0xFFFFFF.
    #[inline]
    fn write_u24_le(&mut self, value: u32) -> Result<()> {
        // Could add a check: if value > 0xFFFFFF { return Err(...) }
        self.put_uint_le(value as u64, 3); // Write lower 3 bytes
        Ok(())
    }

    /// Writes an `i32` as a little-endian 24-bit signed integer (triad).
    /// Value must be within signed 24-bit range.
    #[inline]
    fn write_i24_le(&mut self, value: i32) -> Result<()> {
        // Could add range check
        self.put_uint_le(value as u64, 3); // Write lower 3 bytes (relies on two's complement)
        Ok(())
    }


    /// Writes a `u32` in little-endian format.
    #[inline]
    fn write_u32_le(&mut self, value: u32) -> Result<()> {
        self.put_u32_le(value);
        Ok(())
    }

    /// Writes an `i32` in little-endian format.
    #[inline]
    fn write_i32_le(&mut self, value: i32) -> Result<()> {
        self.put_i32_le(value);
        Ok(())
    }

    /// Writes a `u64` in little-endian format.
    #[inline]
    fn write_u64_le(&mut self, value: u64) -> Result<()> {
        self.put_u64_le(value);
        Ok(())
    }

    /// Writes an `i64` in little-endian format.
    #[inline]
    fn write_i64_le(&mut self, value: i64) -> Result<()> {
        self.put_i64_le(value);
        Ok(())
    }

    /// Writes a `u128` in little-endian format.
    #[inline]
    fn write_u128_le(&mut self, value: u128) -> Result<()> {
        self.put_u128_le(value);
        Ok(())
    }

    /// Writes an `i128` in little-endian format.
    #[inline]
    fn write_i128_le(&mut self, value: i128) -> Result<()> {
        self.put_i128_le(value);
        Ok(())
    }

    // --- Multi-byte Integers (Big Endian) ---

    /// Writes a `u16` in big-endian format.
    #[inline]
    fn write_u16_be(&mut self, value: u16) -> Result<()> {
        self.put_u16(value);
        Ok(())
    }

    /// Writes an `i16` in big-endian format.
    #[inline]
    fn write_i16_be(&mut self, value: i16) -> Result<()> {
        self.put_i16(value);
        Ok(())
    }

    /// Writes a `u32` as a big-endian 24-bit unsigned integer (triad).
    /// Value must be <= 0xFFFFFF.
    #[inline]
    fn write_u24_be(&mut self, value: u32) -> Result<()> {
        self.put_uint(value as u64, 3); // Write lower 3 bytes
        Ok(())
    }

    /// Writes an `i32` as a big-endian 24-bit signed integer (triad).
    /// Value must be within signed 24-bit range.
    #[inline]
    fn write_i24_be(&mut self, value: i32) -> Result<()> {
        self.put_uint(value as u64, 3); // Write lower 3 bytes (relies on two's complement)
        Ok(())
    }

    /// Writes a `u32` in big-endian format.
    #[inline]
    fn write_u32_be(&mut self, value: u32) -> Result<()> {
        self.put_u32(value);
        Ok(())
    }

    /// Writes an `i32` in big-endian format.
    #[inline]
    fn write_i32_be(&mut self, value: i32) -> Result<()> {
        self.put_i32(value);
        Ok(())
    }

    /// Writes a `u64` in big-endian format.
    #[inline]
    fn write_u64_be(&mut self, value: u64) -> Result<()> {
        self.put_u64(value);
        Ok(())
    }

    /// Writes an `i64` in big-endian format.
    #[inline]
    fn write_i64_be(&mut self, value: i64) -> Result<()> {
        self.put_i64(value);
        Ok(())
    }

    /// Writes a `u128` in big-endian format.
    #[inline]
    fn write_u128_be(&mut self, value: u128) -> Result<()> {
        self.put_u128(value);
        Ok(())
    }

    /// Writes an `i128` in big-endian format.
    #[inline]
    fn write_i128_be(&mut self, value: i128) -> Result<()> {
        self.put_i128(value);
        Ok(())
    }

    // --- Floating Point (Little Endian) ---

    /// Writes an `f32` in little-endian format.
    #[inline]
    fn write_f32_le(&mut self, value: f32) -> Result<()> {
        self.put_f32_le(value);
        Ok(())
    }

    /// Writes an `f64` in little-endian format.
    #[inline]
    fn write_f64_le(&mut self, value: f64) -> Result<()> {
        self.put_f64_le(value);
        Ok(())
    }

    // --- Floating Point (Big Endian) ---

    /// Writes an `f32` in big-endian format.
    #[inline]
    fn write_f32_be(&mut self, value: f32) -> Result<()> {
        self.put_f32(value);
        Ok(())
    }

    /// Writes an `f64` in big-endian format.
    #[inline]
    fn write_f64_be(&mut self, value: f64) -> Result<()> {
        self.put_f64(value);
        Ok(())
    }

    // --- Variable Length Integers ---

    /// Writes an unsigned 32-bit variable-length integer (VarInt).
    fn write_varu32(&mut self, mut value: u32) -> Result<()> {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80; // Set continuation bit
            }
            self.put_u8(byte);
            if value == 0 {
                return Ok(());
            }
            // We don't explicitly check for MAX_BYTES here, as u32 naturally limits it.
        }
    }

    /// Writes a signed 32-bit variable-length integer (VarInt), using ZigZag encoding.
    #[inline]
    fn write_vari32(&mut self, value: i32) -> Result<()> {
        // ZigZag encode: (n << 1) ^ (n >> 31)
        let unsigned = (value << 1) ^ (value >> 31);
        self.write_varu32(unsigned as u32)
    }

    /// Writes an unsigned 64-bit variable-length integer (VarLong).
    fn write_varu64(&mut self, mut value: u64) -> Result<()> {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80; // Set continuation bit
            }
            self.put_u8(byte);
            if value == 0 {
                return Ok(());
            }
            // VarLongs can technically exceed 10 bytes if not careful,
            // but the u64 type prevents infinite loops.
            // No explicit MAX_BYTES check needed here for correctness,
            // but could add for strict protocol adherence if needed.
        }
    }

    /// Writes a signed 64-bit variable-length integer (VarLong), using ZigZag encoding.
    #[inline]
    fn write_vari64(&mut self, value: i64) -> Result<()> {
        // ZigZag encode: (n << 1) ^ (n >> 63)
        let unsigned = (value << 1) ^ (value >> 63);
        self.write_varu64(unsigned as u64)
    }

    // --- Slices and Strings ---

    /// Writes a byte slice prefixed with a `VarInt` length.
    fn write_bytes_varint_len(&mut self, bytes: &[u8]) -> Result<()> {
        let len = u32::try_from(bytes.len()).map_err(|_| BinaryError::InvalidData("Byte slice length exceeds u32::MAX".to_string()))?;
        self.write_varu32(len)?;
        self.put_slice(bytes);
        Ok(())
    }

    /// Writes a UTF-8 string prefixed with a `VarInt` length.
    fn write_string_varint_len(&mut self, string: &str) -> Result<()> {
        self.write_bytes_varint_len(string.as_bytes())
    }

    /// Writes a raw byte slice (without length prefix).
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.put_slice(bytes);
        Ok(())
    }

    // --- Complex Types ---

    /// Writes a `Uuid` (16 bytes) in little-endian format (standard for MCBE).
    fn write_uuid_le(&mut self, uuid: &Uuid) -> Result<()> {
        // Use the uuid crate's built-in LE representation
        self.put_slice(uuid.to_bytes_le().as_slice());
        Ok(())
    }

    /// Writes a `Uuid` (16 bytes) in big-endian format.
    fn write_uuid_be(&mut self, uuid: &Uuid) -> Result<()> {
        // Use the uuid crate's default BE representation
        self.put_slice(uuid.as_bytes());
        Ok(())
    }

}

// Implement the trait for all types that implement `BufMut`.
impl<T: BufMut> BinaryWritter for T {}


// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};

    #[test]
    fn test_u8_i8_bool() {
        let mut writer = BytesMut::new();
        writer.write_u8(0xAB).unwrap();
        writer.write_i8(-5).unwrap(); // 0xFB
        writer.write_bool(true).unwrap(); // 0x01
        writer.write_bool(false).unwrap(); // 0x00

        let mut reader = writer.freeze(); // Convert to Bytes for reading
        assert_eq!(reader.read_u8().unwrap(), 0xAB);
        assert_eq!(reader.read_i8().unwrap(), -5);
        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(matches!(reader.read_u8(), Err(BinaryError::UnexpectedEof { needed: 1, remaining: 0 })));
    }

    #[test]
    fn test_endianness_u16() {
        let value: u16 = 0xABCD;
        // LE: CD AB
        // BE: AB CD
        let mut writer_le = BytesMut::new();
        writer_le.write_u16_le(value).unwrap();
        assert_eq!(writer_le.as_ref(), &[0xCD, 0xAB]);
        let mut reader_le = writer_le.freeze();
        assert_eq!(reader_le.read_u16_le().unwrap(), value);

        let mut writer_be = BytesMut::new();
        writer_be.write_u16_be(value).unwrap();
        assert_eq!(writer_be.as_ref(), &[0xAB, 0xCD]);
        let mut reader_be = writer_be.freeze();
        assert_eq!(reader_be.read_u16_be().unwrap(), value);
    }

    #[test]
    fn test_endianness_i32() {
        let value: i32 = -10_000_000; // 0xFF67_6980
        // LE: 80 69 67 FF
        // BE: FF 67 69 80
        let mut writer_le = BytesMut::new();
        writer_le.write_i32_le(value).unwrap();
        assert_eq!(writer_le.as_ref(), &[0x80, 0x69, 0x67, 0xFF]);
        let mut reader_le = writer_le.freeze();
        assert_eq!(reader_le.read_i32_le().unwrap(), value);

        let mut writer_be = BytesMut::new();
        writer_be.write_i32_be(value).unwrap();
        assert_eq!(writer_be.as_ref(), &[0xFF, 0x67, 0x69, 0x80]);
        let mut reader_be = writer_be.freeze();
        assert_eq!(reader_be.read_i32_be().unwrap(), value);
    }


    #[test]
    fn test_triads() {
        let u_value: u32 = 0xABCDEF;
        let i_value_pos: i32 = 0x7BCDEF;
        let i_value_neg: i32 = -0x800000; // Smallest signed 24-bit
        // LE representation: 0x800000 -> 00 00 80
        // BE representation: 0x800000 -> 80 00 00

        // --- LE ---
        let mut writer_le = BytesMut::new();
        writer_le.write_u24_le(u_value).unwrap(); // EF CD AB
        writer_le.write_i24_le(i_value_pos).unwrap(); // EF CD 7B
        writer_le.write_i24_le(i_value_neg).unwrap(); // 00 00 80

        let mut reader_le = writer_le.freeze();
        assert_eq!(reader_le.read_u24_le().unwrap(), u_value);
        assert_eq!(reader_le.read_i24_le().unwrap(), i_value_pos);
        assert_eq!(reader_le.read_i24_le().unwrap(), i_value_neg);

        // --- BE ---
        let mut writer_be = BytesMut::new();
        writer_be.write_u24_be(u_value).unwrap(); // AB CD EF
        writer_be.write_i24_be(i_value_pos).unwrap(); // 7B CD EF
        writer_be.write_i24_be(i_value_neg).unwrap(); // 80 00 00

        let mut reader_be = writer_be.freeze();
        assert_eq!(reader_be.read_u24_be().unwrap(), u_value);
        assert_eq!(reader_be.read_i24_be().unwrap(), i_value_pos);
        assert_eq!(reader_be.read_i24_be().unwrap(), i_value_neg);

    }


    #[test]
    fn test_floats() {
        let f32_val: f32 = 123.456;
        let f64_val: f64 = -987.654e10;

        let mut writer = BytesMut::new();
        writer.write_f32_le(f32_val).unwrap();
        writer.write_f64_be(f64_val).unwrap();

        let mut reader = writer.freeze();
        assert_eq!(reader.read_f32_le().unwrap(), f32_val);
        assert_eq!(reader.read_f64_be().unwrap(), f64_val);
    }

    #[test]
    fn test_varint_u32() {
        let values = [0u32, 1, 127, 128, 16383, 16384, 2097151, 2097152, 268435455, 268435456, u32::MAX];
        let expected_encodings = [
            vec![0x00], vec![0x01], vec![0x7f], vec![0x80, 0x01], vec![0xff, 0x7f], vec![0x80, 0x80, 0x01],
            vec![0xff, 0xff, 0x7f], vec![0x80, 0x80, 0x80, 0x01], vec![0xff, 0xff, 0xff, 0x7f],
            vec![0x80, 0x80, 0x80, 0x80, 0x01], vec![0xff, 0xff, 0xff, 0xff, 0x0f], // u32::MAX
        ];

        for (i, &value) in values.iter().enumerate() {
            let mut writer = BytesMut::new();
            writer.write_varu32(value).unwrap();
            assert_eq!(writer.as_ref(), expected_encodings[i].as_slice(), "Encoding failed for {}", value);

            let mut reader = writer.freeze();
            assert_eq!(reader.read_varu32().unwrap(), value, "Decoding failed for {}", value);
            assert!(reader.is_empty(), "Reader not empty after decoding {}", value);
        }
    }

    #[test]
    fn test_varint_i32() {
        let values = [0i32, -1, 1, -2, 2, i32::MAX, i32::MIN];
        // ZigZag: 0->0, -1->1, 1->2, -2->3, 2->4, MAX->MAX*2, MIN->MAX*2+1
        let expected_unsigned = [0u32, 1, 2, 3, 4, (i32::MAX as u32) << 1, (i32::MAX as u32) << 1 | 1];
        let expected_encodings = [
            vec![0x00], vec![0x01], vec![0x02], vec![0x03], vec![0x04],
            vec![0xfe, 0xff, 0xff, 0xff, 0x0f], // ZigZag(i32::MAX)
            vec![0xff, 0xff, 0xff, 0xff, 0x0f], // ZigZag(i32::MIN)
        ];

        for (i, &value) in values.iter().enumerate() {
            // Test ZigZag encoding directly
            let unsigned_zigzag = (value << 1) ^ (value >> 31);
            assert_eq!(unsigned_zigzag as u32, expected_unsigned[i], "ZigZag encoding mismatch for {}", value);

            let mut writer = BytesMut::new();
            writer.write_vari32(value).unwrap();
            assert_eq!(writer.as_ref(), expected_encodings[i].as_slice(), "Encoding failed for {}", value);

            let mut reader = writer.freeze();
            assert_eq!(reader.read_vari32().unwrap(), value, "Decoding failed for {}", value);
            assert!(reader.is_empty(), "Reader not empty after decoding {}", value);
        }
    }

    #[test]
    fn test_varint_u64() {
        let values = [0u64, 1, 127, 128, u64::MAX];
        let expected_encodings = [
            vec![0x00], vec![0x01], vec![0x7f], vec![0x80, 0x01],
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01], // u64::MAX
        ];

        for (i, &value) in values.iter().enumerate() {
            let mut writer = BytesMut::new();
            writer.write_varu64(value).unwrap();
            assert_eq!(writer.as_ref(), expected_encodings[i].as_slice(), "Encoding failed for {}", value);

            let mut reader = writer.freeze();
            assert_eq!(reader.read_varu64().unwrap(), value, "Decoding failed for {}", value);
            assert!(reader.is_empty(), "Reader not empty after decoding {}", value);
        }
    }

    #[test]
    fn test_varint_i64() {
        let values = [0i64, -1, 1, i64::MAX, i64::MIN];
        // ZigZag: 0->0, -1->1, 1->2, MAX->MAX*2, MIN->MAX*2+1
        let expected_unsigned = [0u64, 1, 2, (i64::MAX as u64) << 1, (i64::MAX as u64) << 1 | 1];
        let expected_encodings = [
            vec![0x00], vec![0x01], vec![0x02],
            vec![0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01], // ZigZag(i64::MAX)
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01], // ZigZag(i64::MIN)
        ];

        for (i, &value) in values.iter().enumerate() {
            // Test ZigZag encoding directly
            let unsigned_zigzag = (value << 1) ^ (value >> 63);
            assert_eq!(unsigned_zigzag as u64, expected_unsigned[i], "ZigZag encoding mismatch for {}", value);

            let mut writer = BytesMut::new();
            writer.write_vari64(value).unwrap();
            assert_eq!(writer.as_ref(), expected_encodings[i].as_slice(), "Encoding failed for {}", value);

            let mut reader = writer.freeze();
            assert_eq!(reader.read_vari64().unwrap(), value, "Decoding failed for {}", value);
            assert!(reader.is_empty(), "Reader not empty after decoding {}", value);
        }
    }

    #[test]
    fn test_varint_errors() {
        // Too long (u32)
        let mut too_long_u32 = Bytes::from_static(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);
        assert!(matches!(too_long_u32.read_varu32(), Err(BinaryError::VarIntTooLong { max_bytes: 5 })));

        // Too long (u64)
        let mut too_long_u64 = Bytes::from_static(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);
        assert!(matches!(too_long_u64.read_varu64(), Err(BinaryError::VarIntTooLong { max_bytes: 10 })));

        // EOF during read
        let mut eof_u32 = Bytes::from_static(&[0x80, 0x80]);
        assert!(matches!(eof_u32.read_varu32(), Err(BinaryError::UnexpectedEof { .. })));

        // Overlong encoding u32 (5th byte > 0x0F)
        let mut overlong_u32 = Bytes::from_static(&[0xff, 0xff, 0xff, 0xff, 0x1f]); // Value would fit, but encoding invalid
        assert!(matches!(overlong_u32.read_varu32(), Err(BinaryError::VarIntOutOfRange)));

        // Overlong encoding u64 (10th byte > 0x01)
        let mut overlong_u64 = Bytes::from_static(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02]);
        assert!(matches!(overlong_u64.read_varu64(), Err(BinaryError::VarIntOutOfRange)));
    }

    #[test]
    fn test_string_and_bytes() {
        let test_string = "Hello, Amethyst! 💜";
        let test_bytes = &[0xDE, 0xAD, 0xBE, 0xEF];

        let mut writer = BytesMut::new();
        writer.write_string_varint_len(test_string).unwrap();
        writer.write_bytes_varint_len(test_bytes).unwrap();
        writer.write_bytes(&[0xCA, 0xFE]).unwrap(); // Raw bytes

        let mut reader = writer.freeze();
        assert_eq!(reader.read_string_varint_len().unwrap(), test_string);
        assert_eq!(reader.read_bytes_varint_len().unwrap().as_ref(), test_bytes);
        assert_eq!(reader.read_bytes(2).unwrap().as_ref(), &[0xCA, 0xFE]);
        assert!(reader.is_empty());
    }

    #[test]
    fn test_uuid() {
        let uuid_val = Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8").unwrap();
        // LE Bytes: a4 a3 a2 a1 b2 b1 c2 c1 d1 d2 d3 d4 d5 d6 d7 d8
        // BE Bytes: a1 a2 a3 a4 b1 b2 c1 c2 d1 d2 d3 d4 d5 d6 d7 d8 (Standard UUID format)

        // Test LE
        let mut writer_le = BytesMut::new();
        writer_le.write_uuid_le(&uuid_val).unwrap();
        assert_eq!(writer_le.len(), 16);

        let mut reader_le = writer_le.freeze();
        let read_uuid_le = reader_le.read_uuid_le().unwrap();
        assert_eq!(read_uuid_le, uuid_val);

        // Test BE
        let mut writer_be = BytesMut::new();
        writer_be.write_uuid_be(&uuid_val).unwrap();
        assert_eq!(writer_be.len(), 16);
        // Verify bytes manually if needed:
        // assert_eq!(writer_be.as_ref(), &[0xa1, 0xa2, ...]);


        let mut reader_be = writer_be.freeze();
        let read_uuid_be = reader_be.read_uuid_be().unwrap();
        assert_eq!(read_uuid_be, uuid_val);
    }

    #[test]
    fn test_remaining_bytes() {
        let data = &[1, 2, 3, 4, 5];
        let mut reader = Bytes::from_static(data);

        assert_eq!(reader.read_u8().unwrap(), 1);
        let remaining = reader.read_remaining_bytes();
        assert_eq!(remaining.as_ref(), &[2, 3, 4, 5]);
        assert!(reader.is_empty()); // read_remaining_bytes consumes all
    }
}