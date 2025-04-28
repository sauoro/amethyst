use crate::error::BinaryError;
use crate::io::{BinaryReader, BinaryWriter};
use std::net::SocketAddr;

const MAX_VEC_LEN: u32 = 1_000_000;

pub trait Readable: Sized {
    /// Reads an instance from the reader.
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError>;
}

pub trait Writable {
    /// Writes this instance to the writer.
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError>;
}

macro_rules! impl_primitive_readable {
    ($($ty:ty => $method:ident),*) => {
        $(
            impl Readable for $ty {
                #[inline]
                fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
                    reader.$method()
                }
            }
        )*
    };
}

macro_rules! impl_primitive_writable {
    ($($ty:ty => $method:ident),*) => {
        $(
            impl Writable for $ty {
                #[inline]
                fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
                    writer.$method(*self)
                }
            }
        )*
    };
}

impl_primitive_readable! {
    u8 => read_u8, i8 => read_i8,
    u16 => read_u16, i16 => read_i16,
    u32 => read_u32, i32 => read_i32,
    u64 => read_u64, i64 => read_i64,
    u128 => read_u128, i128 => read_i128,
    f32 => read_f32, f64 => read_f64,
    bool => read_bool
}

impl_primitive_writable! {
    u8 => write_u8, i8 => write_i8,
    u16 => write_u16, i16 => write_i16,
    u32 => write_u32, i32 => write_i32,
    u64 => write_u64, i64 => write_i64,
    u128 => write_u128, i128 => write_i128,
    f32 => write_f32, f64 => write_f64,
    bool => write_bool
}

impl Readable for String {
    #[inline]
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        reader.read_string()
    }
}

impl Writable for String {
    #[inline]
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_string(self)
    }
}

impl Writable for &str {
    #[inline]
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_string(self)
    }
}

impl<T: Readable> Readable for Option<T> {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        if reader.read_bool()? {
            Ok(Some(T::read(reader)?))
        } else {
            Ok(None)
        }
    }
}

impl<T: Writable> Writable for Option<T> {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        match self {
            Some(value) => {
                writer.write_bool(true)?;
                value.write(writer)?;
            }
            None => writer.write_bool(false)?,
        }
        Ok(())
    }
}

impl<T: Readable> Readable for Vec<T> {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let len = reader.read_var_u32()?;
        if len > MAX_VEC_LEN {
            return Err(BinaryError::InvalidLength {
                expected: len,
                max: MAX_VEC_LEN as usize,
            });
        }
        let len = len as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::read(reader)?);
        }
        Ok(vec)
    }
}

impl<T: Writable> Writable for Vec<T> {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        if self.len() > u32::MAX as usize {
            return Err(BinaryError::Overflow);
        }
        writer.write_var_u32(self.len() as u32)?;
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

impl Readable for SocketAddr {
    #[inline]
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        reader.read_socket_addr()
    }
}

impl Writable for SocketAddr {
    #[inline]
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_socket_addr(self)
    }
}

impl<T: Readable, const N: usize> Readable for [T; N] {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let mut vec = Vec::with_capacity(N);
        for _ in 0..N {
            vec.push(T::read(reader)?);
        }
        vec.try_into().map_err(|_| BinaryError::Custom("Array conversion failed".into()))
    }
}

impl<T: Writable, const N: usize> Writable for [T; N] {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

impl<T: Readable, U: Readable> Readable for (T, U) {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        Ok((T::read(reader)?, U::read(reader)?))
    }
}

impl<T: Writable, U: Writable> Writable for (T, U) {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        self.0.write(writer)?;
        self.1.write(writer)?;
        Ok(())
    }
}
