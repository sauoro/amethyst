use crate::packet::{Packet, CONNECTED_PING, CONNECTED_PONG, DISCONNECTION_NOTIFICATION};
use binary::{BinaryReader, BinaryResult, BinaryWriter};

#[derive(Debug, Clone)]
pub struct ConnectedPing {
    pub client_timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct ConnectedPong {
    pub client_timestamp: i64,
    pub server_timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct DisconnectionNotification;

impl Packet for ConnectedPing {
    fn id() -> u8 {
        CONNECTED_PING
    }

    fn serialize(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_u8(Self::id())?;
        writer.write_i64_be(self.client_timestamp)?;
        Ok(())
    }

    fn deserialize(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        reader.read_u8()?;
        let client_timestamp = reader.read_i64_be()?;
        Ok(Self { client_timestamp })
    }
}

impl Packet for ConnectedPong {
    fn id() -> u8 {
        CONNECTED_PONG
    }

    fn serialize(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_u8(Self::id())?;
        writer.write_i64_be(self.client_timestamp)?;
        writer.write_i64_be(self.server_timestamp)?;
        Ok(())
    }

    fn deserialize(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        reader.read_u8()?;
        let client_timestamp = reader.read_i64_be()?;
        let server_timestamp = reader.read_i64_be()?;
        Ok(Self {
            client_timestamp,
            server_timestamp,
        })
    }
}

impl Packet for DisconnectionNotification {
    fn id() -> u8 {
        DISCONNECTION_NOTIFICATION
    }
    fn serialize(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_u8(Self::id())?;
        Ok(())
    }

    fn deserialize(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        reader.read_u8()?;
        Ok(Self)
    }
}
