use crate::packet::{Packet, UNCONNECTED_PING, UNCONNECTED_PONG};
use binary::{BinaryReader, BinaryResult, BinaryWriter};

#[derive(Debug, Clone)]
pub struct UnconnectedPing {
    pub client_timestamp: i64,
    pub client_guid: u64,
}

#[derive(Debug, Clone)]
pub struct UnconnectedPong {
    pub server_timestamp: i64,
    pub server_guid: u64,
    pub server_name: String,
}

impl Packet for UnconnectedPing {
    fn id() -> u8 {
        UNCONNECTED_PING
    }
    
    fn serialize(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_u8(Self::id())?;
        writer.write_i64_be(self.client_timestamp)?;
        writer.write_magic()?;
        writer.write_u64_be(self.client_guid)?;
        Ok(())
    }
    
    fn deserialize(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        reader.read_u8();
        let client_timestamp = reader.read_i64_be()?;
        if !reader.read_magic()? {
            return Err(binary::BinaryError::InvalidData(
                "Invalid magic sequence in UnconnectedPing".into(),
            ));
        }
        let client_guid = reader.read_u64_be()?;
        Ok(Self { client_timestamp, client_guid })
    }
}

impl Packet for UnconnectedPong {
    fn id() -> u8 {
        UNCONNECTED_PONG
    }
    
    fn serialize(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_u8(Self::id())?;
        writer.write_i64_be(self.server_timestamp)?;
        writer.write_u64_be(self.server_guid)?;
        writer.write_magic()?;
        writer.write_string(&self.server_name)?;
        Ok(())
    }
    
    fn deserialize(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        reader.read_u8()?;
        let server_timestamp = reader.read_i64_be()?;
        let server_guid = reader.read_u64_be()?;
        if !reader.read_magic()? {
            return Err(binary::BinaryError::InvalidData(
                "Invalid magic sequence in UnconnectedPong".into(),
            ))
        }
        let server_name = reader.read_string()?;
        Ok(Self { server_timestamp, server_guid, server_name })
    }
}
