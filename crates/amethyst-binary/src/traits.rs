use crate::error::BinaryError;
use crate::io::{BinaryReader, BinaryWriter};
use std::net::SocketAddr;

/// Trait for types that can be read from a `BinaryReader`.
pub trait Readable: Sized {
    /// Reads an instance of `Self` from the reader.
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError>;
}

/// Trait for types that can be written to a `BinaryWriter`.
pub trait Writable {
    /// Writes this instance to the writer.
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError>;
}

impl<T: Readable> Readable for Result<T, BinaryError> {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        Ok(T::read(reader))
    }
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

// String
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

// &str (only Writable)
impl Writable for &str {
    #[inline]
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_string(self)
    }
}

// Option<T>
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
        if let Some(value) = self {
            writer.write_bool(true)?;
            value.write(writer)?;
        } else {
            writer.write_bool(false)?;
        }
        Ok(())
    }
}

// Vec<T> (using VarUInt32 for length)
impl<T: Readable> Readable for Vec<T> {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let len = reader.read_var_u32()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::read(reader)?);
        }
        Ok(vec)
    }
}

impl<T: Writable> Writable for Vec<T> {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_var_u32(self.len() as u32)?; // Error if len > u32::MAX
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
