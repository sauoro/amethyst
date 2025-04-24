use crate::utils::{BinaryError, BinaryReader, BinaryResult, BinaryWriter};

pub struct PacketId {}

impl PacketId {
    pub const CONNECTED_PING: u8 = 0x00;
    pub const UNCONNECTED_PING: u8 = 0x01;
}

pub trait Packet: Sized {
    fn id() -> u8;
    fn serialize(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()>;
    fn deserialize(reader: &mut impl BinaryReader) -> BinaryResult<Self>;
}

macro_rules! implement_packet {
    ($struct_name:ident, $packet_id:expr) => {
        impl Packet for $struct_name {
            fn id() -> u8 {
                $packet_id
            }

            #[inline]
            fn serialize(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
                writer.write_u8(Self::id())?;
                self.encode(writer)
            }

            #[inline]
            fn deserialize(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
                let id = reader.read_u8()?;
                if id != Self::id() {
                    return Err(BinaryError::InvalidData(format!(
                        "Invalid Packet ID for {}: expected {}, got {}",
                        stringify!($struct_name),
                        Self::id(),
                        id
                    )));
                }
                Self::decode(reader)
            }
        }
    };
}

pub struct ConnectedPing {
    pub client_timestamp: i64,
}

impl ConnectedPing {
    pub fn encode(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_i64_be(self.client_timestamp)?;
        Ok(())
    }

    pub fn decode(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        let client_timestamp = reader.read_i64_be()?;
        Ok(Self { client_timestamp })
    }
}

implement_packet!(ConnectedPing, PacketId::CONNECTED_PING);

pub struct UnconnectedPing {
    pub client_timestamp: i64,
    pub client_guid: u64,
}

impl UnconnectedPing {
    pub fn encode(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_i64_be(self.client_timestamp)?;
        writer.write_magic()?;
        writer.write_u64_be(self.client_guid)?;
        Ok(())
    }

    pub fn decode(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        let client_timestamp = reader.read_i64_be()?;
        if !reader.read_magic()? {
            return Err(BinaryError::InvalidData(
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
