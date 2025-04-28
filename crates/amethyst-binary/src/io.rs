use bytes::{Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct BinaryReader {
    buffer: Bytes
}

#[derive(Debug, Clone, Default)]
pub struct BinaryWriter {
    buffer: BytesMut,
}