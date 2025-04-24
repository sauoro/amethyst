use crate::packet::implement_packet;
use crate::packet::PacketId;
use binary::{BinaryReader, BinaryResult, BinaryWriter};

#[derive(Debug, Clone)]
pub struct ConnectedPing {
    pub client_timestamp: i64,
}

impl ConnectedPing {
    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_i64_be(self.client_timestamp)?;
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        let client_timestamp = reader.read_i64_be()?;
        Ok(Self { client_timestamp })
    }
}
implement_packet!(ConnectedPing, PacketId::CONNECTED_PING);

#[derive(Debug, Clone)]
pub struct ConnectedPong {
    pub client_timestamp: i64,
    pub server_timestamp: i64,
}
impl ConnectedPong {
    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_i64_be(self.client_timestamp)?;
        writer.write_i64_be(self.server_timestamp)?;
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        let client_timestamp = reader.read_i64_be()?;
        let server_timestamp = reader.read_i64_be()?;
        Ok(Self {
            client_timestamp,
            server_timestamp,
        })
    }
}
implement_packet!(ConnectedPong, PacketId::CONNECTED_PONG);

#[derive(Debug, Clone)]
pub struct DisconnectionNotification;

impl DisconnectionNotification {
    pub fn encode_payload(&self, _writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        Ok(())
    }

    pub fn decode_payload(_reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        Ok(Self)
    }
}
implement_packet!(DisconnectionNotification, PacketId::DISCONNECTION_NOTIFICATION);