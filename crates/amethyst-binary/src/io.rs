use crate::error::BinaryError;
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct BinaryReader {
    buffer: Bytes,
}

impl BinaryReader {
    /// Creates a new `BinaryReader` wrapping the given `Bytes`.
    #[inline]
    pub fn new(buffer: Bytes) -> Self {
        Self { buffer }
    }

    /// Creates a new `BinaryReader` from a byte slice.
    ///
    /// This involves a copy of the slice data.
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            buffer: Bytes::copy_from_slice(slice),
        }
    }

    /// Returns the number of bytes remaining in the buffer.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.buffer.remaining()
    }

    /// Returns `true` if there are no bytes remaining in the buffer.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Advances the internal cursor by `cnt` bytes.
    ///
    /// # Errors
    ///
    /// Returns `Err(BinaryError::UnexpectedEOF)` if `cnt` is greater than the
    /// number of remaining bytes.
    #[inline]
    pub fn advance(&mut self, cnt: usize) -> Result<(), BinaryError> {
        if self.remaining() >= cnt {
            self.buffer.advance(cnt);
            Ok(())
        } else {
            Err(BinaryError::UnexpectedEOF)
        }
    }

    /// Peeks at the next byte without advancing the cursor.
    ///
    /// # Errors
    ///
    /// Returns `Err(BinaryError::UnexpectedEOF)` if the buffer is empty.
    #[inline]
    pub fn peek_u8(&self) -> Result<u8, BinaryError> {
        if self.remaining() >= 1 {
            Ok(self.buffer.chunk()[0])
        } else {
            Err(BinaryError::UnexpectedEOF)
        }
    }

    /// Reads a single byte (`u8`).
    #[inline]
    pub fn read_u8(&mut self) -> Result<u8, BinaryError> {
        if self.remaining() >= 1 {
            Ok(self.buffer.get_u8())
        } else {
            Err(BinaryError::UnexpectedEOF)
        }
    }

    /// Reads a single signed byte (`i8`).
    #[inline]
    pub fn read_i8(&mut self) -> Result<i8, BinaryError> {
        if self.remaining() >= 1 {
            Ok(self.buffer.get_i8())
        } else {
            Err(BinaryError::UnexpectedEOF)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BinaryWriter {
    buffer: BytesMut,
}

impl BinaryWriter {
    /// Creates a new empty `BinaryWriter`.
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
        }
    }

    /// Creates a new `BinaryWriter` with a specified initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
        }
    }

    /// Returns the current length of the buffer in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the current capacity of the buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Clears the buffer, removing all data. Capacity is preserved.
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Consumes the writer and returns the underlying `BytesMut` buffer.
    #[inline]
    pub fn into_inner(self) -> BytesMut {
        self.buffer
    }

    /// Consumes the writer and returns an immutable `Bytes` buffer.
    #[inline]
    pub fn freeze(self) -> Bytes {
        self.buffer.freeze()
    }

    /// Returns a reference to the written bytes as a slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[..]
    }

    /// Writes a single byte (`u8`).
    #[inline]
    pub fn write_u8(&mut self, value: u8) -> Result<(), BinaryError> {
        self.buffer.put_u8(value);
        Ok(())
    }

    /// Writes a single signed byte (`i8`).
    #[inline]
    pub fn write_i8(&mut self, value: i8) -> Result<(), BinaryError> {
        self.buffer.put_i8(value);
        Ok(())
    }
}
