use crate::error::BinaryError;
use crate::error::BinaryError::{InvalidData, UnexpectedEOF};
use crate::traits::{Readable, Writable};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

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

    /// Creates a new `BinaryReader` from a byte slice (involves a copy).
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            buffer: Bytes::copy_from_slice(slice),
        }
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.buffer.remaining()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[inline]
    pub fn advance(&mut self, cnt: usize) -> Result<(), BinaryError> {
        if self.remaining() >= cnt {
            self.buffer.advance(cnt);
            Ok(())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn peek_u8(&self) -> Result<u8, BinaryError> {
        if self.remaining() >= 1 {
            Ok(self.buffer.chunk()[0])
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_u8(&mut self) -> Result<u8, BinaryError> {
        if self.remaining() >= 1 {
            Ok(self.buffer.get_u8())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_i8(&mut self) -> Result<i8, BinaryError> {
        if self.remaining() >= 1 {
            Ok(self.buffer.get_i8())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_u16(&mut self) -> Result<u16, BinaryError> {
        if self.remaining() >= 2 {
            Ok(self.buffer.get_u16())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_u16_le(&mut self) -> Result<u16, BinaryError> {
        if self.remaining() >= 2 {
            Ok(self.buffer.get_u16_le())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_i16(&mut self) -> Result<i16, BinaryError> {
        if self.remaining() >= 2 {
            Ok(self.buffer.get_i16())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_i16_le(&mut self) -> Result<i16, BinaryError> {
        if self.remaining() >= 2 {
            Ok(self.buffer.get_i16_le())
        } else {
            Err(UnexpectedEOF)
        }
    }

    pub fn read_u24(&mut self) -> Result<u32, BinaryError> {
        if self.remaining() >= 3 {
            let bytes = [
                0,
                self.buffer.get_u8(),
                self.buffer.get_u8(),
                self.buffer.get_u8(),
            ];
            Ok(u32::from_be_bytes(bytes))
        } else {
            Err(UnexpectedEOF)
        }
    }

    pub fn read_u24_le(&mut self) -> Result<u32, BinaryError> {
        if self.remaining() >= 3 {
            let bytes = [
                self.buffer.get_u8(),
                self.buffer.get_u8(),
                self.buffer.get_u8(),
                0,
            ];
            Ok(u32::from_le_bytes(bytes))
        } else {
            Err(UnexpectedEOF)
        }
    }

    pub fn read_i24(&mut self) -> Result<i32, BinaryError> {
        let u = self.read_u24()?;
        if u & 0x00800000 != 0 {
            Ok((u | 0xFF000000) as i32)
        } else {
            Ok(u as i32)
        }
    }

    pub fn read_i24_le(&mut self) -> Result<i32, BinaryError> {
        let u = self.read_u24_le()?;
        if u & 0x00800000 != 0 {
            Ok((u | 0xFF000000) as i32)
        } else {
            Ok(u as i32)
        }
    }

    #[inline]
    pub fn read_u32(&mut self) -> Result<u32, BinaryError> {
        if self.remaining() >= 4 {
            Ok(self.buffer.get_u32())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_u32_le(&mut self) -> Result<u32, BinaryError> {
        if self.remaining() >= 4 {
            Ok(self.buffer.get_u32_le())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_i32(&mut self) -> Result<i32, BinaryError> {
        if self.remaining() >= 4 {
            Ok(self.buffer.get_i32())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_i32_le(&mut self) -> Result<i32, BinaryError> {
        if self.remaining() >= 4 {
            Ok(self.buffer.get_i32_le())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_u64(&mut self) -> Result<u64, BinaryError> {
        if self.remaining() >= 8 {
            Ok(self.buffer.get_u64())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_u64_le(&mut self) -> Result<u64, BinaryError> {
        if self.remaining() >= 8 {
            Ok(self.buffer.get_u64_le())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_i64(&mut self) -> Result<i64, BinaryError> {
        if self.remaining() >= 8 {
            Ok(self.buffer.get_i64())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_i64_le(&mut self) -> Result<i64, BinaryError> {
        if self.remaining() >= 8 {
            Ok(self.buffer.get_i64_le())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_u128(&mut self) -> Result<u128, BinaryError> {
        if self.remaining() >= 16 {
            Ok(self.buffer.get_u128())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_u128_le(&mut self) -> Result<u128, BinaryError> {
        if self.remaining() >= 16 {
            Ok(self.buffer.get_u128_le())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_i128(&mut self) -> Result<i128, BinaryError> {
        if self.remaining() >= 16 {
            Ok(self.buffer.get_i128())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_i128_le(&mut self) -> Result<i128, BinaryError> {
        if self.remaining() >= 16 {
            Ok(self.buffer.get_i128_le())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_f32(&mut self) -> Result<f32, BinaryError> {
        if self.remaining() >= 4 {
            Ok(self.buffer.get_f32())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_f32_le(&mut self) -> Result<f32, BinaryError> {
        if self.remaining() >= 4 {
            Ok(self.buffer.get_f32_le())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_f64(&mut self) -> Result<f64, BinaryError> {
        if self.remaining() >= 8 {
            Ok(self.buffer.get_f64())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_f64_le(&mut self) -> Result<f64, BinaryError> {
        if self.remaining() >= 8 {
            Ok(self.buffer.get_f64_le())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_bool(&mut self) -> Result<bool, BinaryError> {
        Ok(self.read_u8()? != 0)
    }

    #[inline]
    pub fn read_exact(&mut self, dst: &mut [u8]) -> Result<(), BinaryError> {
        let len = dst.len();
        if self.remaining() >= len {
            self.buffer.copy_to_slice(dst);
            Ok(())
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_bytes(&mut self, len: usize) -> Result<Bytes, BinaryError> {
        if self.remaining() >= len {
            Ok(self.buffer.split_to(len))
        } else {
            Err(UnexpectedEOF)
        }
    }

    #[inline]
    pub fn read_remaining(&mut self) -> Bytes {
        self.buffer.split_off(0)
    }

    pub fn read_var_u32(&mut self) -> Result<u32, BinaryError> {
        let mut value: u32 = 0;
        let mut shift: u32 = 0;
        loop {
            let byte = self.read_u8()?;
            value |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
            shift += 7;
            if shift >= 32 {
                return Err(InvalidData("VarInt overflow u32".to_string()));
            }
        }
    }

    pub fn read_var_i32(&mut self) -> Result<i32, BinaryError> {
        let unsigned = self.read_var_u32()?;
        Ok((unsigned >> 1) as i32 ^ -((unsigned & 1) as i32))
    }

    pub fn read_var_u64(&mut self) -> Result<u64, BinaryError> {
        let mut value: u64 = 0;
        let mut shift: u32 = 0;
        loop {
            let byte = self.read_u8()?;
            value |= ((byte & 0x7F) as u64) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
            shift += 7;
            if shift >= 64 {
                return Err(InvalidData("VarLong overflow u64".to_string()));
            }
        }
    }

    pub fn read_var_i64(&mut self) -> Result<i64, BinaryError> {
        let unsigned = self.read_var_u64()?;
        Ok((unsigned >> 1) as i64 ^ -((unsigned & 1) as i64))
    }

    pub fn read_string(&mut self) -> Result<String, BinaryError> {
        let len = self.read_var_u32()? as usize;
        let str_bytes = self.read_bytes(len)?;
        String::from_utf8(str_bytes.to_vec())
            .map_err(|e| InvalidData(format!("Invalid UTF-8 string: {}", e)))
    }

    pub fn read_socket_addr(&mut self) -> Result<SocketAddr, BinaryError> {
        let version = self.read_u8()?;
        match version {
            4 => {
                let mut ip_bytes = [0u8; 4];
                self.read_exact(&mut ip_bytes)?;
                let ip = Ipv4Addr::from(ip_bytes);
                let port = self.read_u16()?;
                Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)))
            }
            6 => {
                let mut ip_bytes = [0u8; 16];
                self.read_exact(&mut ip_bytes)?;
                let ip = Ipv6Addr::from(ip_bytes);
                let port = self.read_u16()?;
                Ok(SocketAddr::V6(SocketAddrV6::new(ip, port, 0, 0)))
            }
            _ => Err(InvalidData("Invalid SocketAddr IP version".to_string())),
        }
    }

    /// Reads a RakNet-specific `SocketAddr` (IPv4 or IPv6).
    pub fn read_raknet_address(&mut self) -> Result<SocketAddr, BinaryError> {
        let ip_ver = self.read_u8()?;
        if ip_ver == 4 {
            if self.remaining() < 6 {
                return Err(UnexpectedEOF);
            }
            let ip = Ipv4Addr::new(
                !self.read_u8()?,
                !self.read_u8()?,
                !self.read_u8()?,
                !self.read_u8()?,
            );
            let port = self.read_u16()?;
            Ok(SocketAddr::new(IpAddr::V4(ip), port))
        } else if ip_ver == 6 {
            if self.remaining() < (2 + 2 + 4 + 16 + 4) {
                return Err(UnexpectedEOF);
            }
            let _family = self.read_i16_le()?;
            let port = self.read_u16()?;
            let flowinfo = self.read_u32()?;
            let mut addr_buf = [0; 16];
            self.read_exact(&mut addr_buf)?;
            let scope_id = self.read_u32()?;
            let ip = Ipv6Addr::from(addr_buf);
            Ok(SocketAddr::V6(SocketAddrV6::new(
                ip, port, flowinfo, scope_id,
            )))
        } else {
            Err(InvalidData("Invalid RakNet SocketAddr IP version".to_string()))
        }
    }

    #[inline]
    pub fn read<T: Readable>(&mut self) -> Result<T, BinaryError> {
        T::read(self)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BinaryWriter {
    buffer: BytesMut,
}

impl BinaryWriter {
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear()
    }

    #[inline]
    pub fn into_inner(self) -> BytesMut {
        self.buffer
    }

    #[inline]
    pub fn freeze(self) -> Bytes {
        self.buffer.freeze()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[..]
    }

    #[inline]
    pub fn write_u8(&mut self, value: u8) -> Result<(), BinaryError> {
        self.buffer.put_u8(value);
        Ok(())
    }

    #[inline]
    pub fn write_i8(&mut self, value: i8) -> Result<(), BinaryError> {
        self.buffer.put_i8(value);
        Ok(())
    }

    #[inline]
    pub fn write_u16(&mut self, value: u16) -> Result<(), BinaryError> {
        self.buffer.put_u16(value);
        Ok(())
    }

    #[inline]
    pub fn write_u16_le(&mut self, value: u16) -> Result<(), BinaryError> {
        self.buffer.put_u16_le(value);
        Ok(())
    }

    #[inline]
    pub fn write_i16(&mut self, value: i16) -> Result<(), BinaryError> {
        self.buffer.put_i16(value);
        Ok(())
    }

    #[inline]
    pub fn write_i16_le(&mut self, value: i16) -> Result<(), BinaryError> {
        self.buffer.put_i16_le(value);
        Ok(())
    }

    pub fn write_u24(&mut self, value: u32) -> Result<(), BinaryError> {
        if value > 0xFFFFFF {
            return Err(InvalidData("Value too large for u24".to_string()));
        }
        let bytes = value.to_be_bytes();
        self.buffer.put_slice(&bytes[1..4]);
        Ok(())
    }

    pub fn write_u24_le(&mut self, value: u32) -> Result<(), BinaryError> {
        if value > 0xFFFFFF {
            return Err(InvalidData("Value too large for u24".to_string()));
        }
        let bytes = value.to_le_bytes();
        self.buffer.put_slice(&bytes[0..3]);
        Ok(())
    }

    pub fn write_i24(&mut self, value: i32) -> Result<(), BinaryError> {
        if !(-0x800000..=0x7FFFFF).contains(&value) {
            return Err(InvalidData("Value out of range for i24".to_string()));
        }
        let bytes = value.to_be_bytes();
        self.buffer.put_slice(&bytes[1..4]);
        Ok(())
    }

    pub fn write_i24_le(&mut self, value: i32) -> Result<(), BinaryError> {
        if !(-0x800000..=0x7FFFFF).contains(&value) {
            return Err(InvalidData("Value out of range for i24".to_string()));
        }
        let bytes = value.to_le_bytes();
        self.buffer.put_slice(&bytes[0..3]);
        Ok(())
    }

    #[inline]
    pub fn write_u32(&mut self, value: u32) -> Result<(), BinaryError> {
        self.buffer.put_u32(value);
        Ok(())
    }

    #[inline]
    pub fn write_u32_le(&mut self, value: u32) -> Result<(), BinaryError> {
        self.buffer.put_u32_le(value);
        Ok(())
    }

    #[inline]
    pub fn write_i32(&mut self, value: i32) -> Result<(), BinaryError> {
        self.buffer.put_i32(value);
        Ok(())
    }

    #[inline]
    pub fn write_i32_le(&mut self, value: i32) -> Result<(), BinaryError> {
        self.buffer.put_i32_le(value);
        Ok(())
    }

    #[inline]
    pub fn write_u64(&mut self, value: u64) -> Result<(), BinaryError> {
        self.buffer.put_u64(value);
        Ok(())
    }

    #[inline]
    pub fn write_u64_le(&mut self, value: u64) -> Result<(), BinaryError> {
        self.buffer.put_u64_le(value);
        Ok(())
    }

    #[inline]
    pub fn write_i64(&mut self, value: i64) -> Result<(), BinaryError> {
        self.buffer.put_i64(value);
        Ok(())
    }

    #[inline]
    pub fn write_i64_le(&mut self, value: i64) -> Result<(), BinaryError> {
        self.buffer.put_i64_le(value);
        Ok(())
    }

    #[inline]
    pub fn write_u128(&mut self, value: u128) -> Result<(), BinaryError> {
        self.buffer.put_u128(value);
        Ok(())
    }

    #[inline]
    pub fn write_u128_le(&mut self, value: u128) -> Result<(), BinaryError> {
        self.buffer.put_u128_le(value);
        Ok(())
    }

    #[inline]
    pub fn write_i128(&mut self, value: i128) -> Result<(), BinaryError> {
        self.buffer.put_i128(value);
        Ok(())
    }

    #[inline]
    pub fn write_i128_le(&mut self, value: i128) -> Result<(), BinaryError> {
        self.buffer.put_i128_le(value);
        Ok(())
    }

    #[inline]
    pub fn write_f32(&mut self, value: f32) -> Result<(), BinaryError> {
        self.buffer.put_f32(value);
        Ok(())
    }

    #[inline]
    pub fn write_f32_le(&mut self, value: f32) -> Result<(), BinaryError> {
        self.buffer.put_f32_le(value);
        Ok(())
    }

    #[inline]
    pub fn write_f64(&mut self, value: f64) -> Result<(), BinaryError> {
        self.buffer.put_f64(value);
        Ok(())
    }

    #[inline]
    pub fn write_f64_le(&mut self, value: f64) -> Result<(), BinaryError> {
        self.buffer.put_f64_le(value);
        Ok(())
    }

    #[inline]
    pub fn write_bool(&mut self, value: bool) -> Result<(), BinaryError> {
        self.buffer.put_u8(value as u8);
        Ok(())
    }

    #[inline]
    pub fn write_bytes(&mut self, src: &[u8]) -> Result<(), BinaryError> {
        self.buffer.put_slice(src);
        Ok(())
    }

    pub fn write_var_u32(&mut self, mut value: u32) -> Result<(), BinaryError> {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.write_u8(byte)?;
            if value == 0 {
                return Ok(());
            }
        }
    }

    pub fn write_var_i32(&mut self, value: i32) -> Result<(), BinaryError> {
        let unsigned = (value << 1) ^ (value >> 31);
        self.write_var_u32(unsigned as u32)
    }

    pub fn write_var_u64(&mut self, mut value: u64) -> Result<(), BinaryError> {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.write_u8(byte)?;
            if value == 0 {
                return Ok(());
            }
        }
    }

    pub fn write_var_i64(&mut self, value: i64) -> Result<(), BinaryError> {
        let unsigned = (value << 1) ^ (value >> 63);
        self.write_var_u64(unsigned as u64)
    }

    pub fn write_string(&mut self, value: &str) -> Result<(), BinaryError> {
        let bytes = value.as_bytes();
        let len = bytes.len();
        self.write_var_u32(len as u32)?;
        self.write_bytes(bytes)
    }

    pub fn write_socket_addr(&mut self, addr: &SocketAddr) -> Result<(), BinaryError> {
        match addr {
            SocketAddr::V4(v4) => {
                self.write_u8(4)?;
                self.write_bytes(&v4.ip().octets())?;
                self.write_u16(v4.port())?;
            }
            SocketAddr::V6(v6) => {
                self.write_u8(6)?;
                self.write_bytes(&v6.ip().octets())?;
                self.write_u16(v6.port())?;
            }
        }
        Ok(())
    }

    /// Writes a RakNet-specific `SocketAddr` (IPv4 or IPv6).
    pub fn write_raknet_address(&mut self, address: SocketAddr) -> Result<(), BinaryError> {
        match address {
            SocketAddr::V4(addr) => {
                self.write_u8(4)?;
                for octet in addr.ip().octets().iter() {
                    self.write_u8(!octet)?;
                }
                self.write_u16(address.port())?;
            }
            SocketAddr::V6(addr) => {
                self.write_u8(6)?;
                self.write_i16_le(23)?;
                self.write_u16(addr.port())?;
                self.write_u32(addr.flowinfo())?;
                self.write_bytes(&addr.ip().octets())?;
                self.write_u32(addr.scope_id())?;
            }
        }
        Ok(())
    }

    #[inline]
    pub fn write<T: Writable + ?Sized>(&mut self, value: &T) -> Result<(), BinaryError> {
        value.write(self)
    }
}

impl From<Vec<u8>> for BinaryReader {
    #[inline]
    fn from(vec: Vec<u8>) -> Self {
        Self::new(Bytes::from(vec))
    }
}

impl From<Bytes> for BinaryReader {
    #[inline]
    fn from(bytes: Bytes) -> Self {
        Self::new(bytes)
    }
}

impl From<BinaryWriter> for BinaryReader {
    #[inline]
    fn from(writer: BinaryWriter) -> Self {
        Self::new(writer.freeze())
    }
}

impl From<Vec<u8>> for BinaryWriter {
    #[inline]
    fn from(vec: Vec<u8>) -> Self {
        Self {
            buffer: BytesMut::from(&vec[..]),
        }
    }
}

impl From<BytesMut> for BinaryWriter {
    #[inline]
    fn from(buffer: BytesMut) -> Self {
        Self { buffer }
    }
}
