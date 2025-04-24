// raknet/src/packet/unconnected.rs
use crate::packet::implement_packet;
use crate::packet::PacketId;
use binary::{BinaryReader, BinaryResult, BinaryWriter};

#[derive(Debug, Clone)]
pub struct UnconnectedPing {
    pub client_timestamp: i64,
    pub client_guid: u64,
}

impl UnconnectedPing {
    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_i64_be(self.client_timestamp)?;
        writer.write_magic()?;
        writer.write_u64_be(self.client_guid)?;
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        let client_timestamp = reader.read_i64_be()?;
        if !reader.read_magic()? {
            return Err(binary::BinaryError::InvalidData(
                "Invalid magic sequence in UnconnectedPing".into(),
            ));
        }
        let client_guid = reader.read_u64_be()?;
        Ok(Self {
            client_timestamp,
            client_guid,
        })
    }
}
implement_packet!(UnconnectedPing, PacketId::UNCONNECTED_PING);

#[derive(Debug, Clone)]
pub struct UnconnectedPong {
    pub server_timestamp: i64,
    pub server_guid: u64,
    pub server_name: String,
}

impl UnconnectedPong {
    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_i64_be(self.server_timestamp)?;
        writer.write_u64_be(self.server_guid)?;
        writer.write_magic()?;
        writer.write_string(&self.server_name)?;
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        let server_timestamp = reader.read_i64_be()?;
        let server_guid = reader.read_u64_be()?;
        if !reader.read_magic()? {
            return Err(binary::BinaryError::InvalidData(
                "Invalid magic sequence in UnconnectedPong".into(),
            ));
        }
        let server_name = reader.read_string()?;
        Ok(Self {
            server_timestamp,
            server_guid,
            server_name,
        })
    }
}
implement_packet!(UnconnectedPong, PacketId::UNCONNECTED_PONG);