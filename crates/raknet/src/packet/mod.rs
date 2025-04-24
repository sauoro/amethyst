use binary::{BinaryReader, BinaryResult, BinaryWriter};

pub mod connected;
pub mod handshake;
pub mod unconnected;
pub mod ack;
pub mod frame;

pub use connected::*;
pub use handshake::*;
pub use unconnected::*;
pub use ack::*;
pub use frame::*;

pub struct PacketId {}

impl PacketId {
    pub const UNCONNECTED_PING: u8 = 0x01;
    pub const UNCONNECTED_PONG: u8 = 0x1c;

    pub const OPEN_CONNECTION_REQUEST_1: u8 = 0x05;
    pub const OPEN_CONNECTION_REPLY_1: u8 = 0x06;
    pub const INCOMPATIBLE_PROTOCOL_VERSION: u8 = 0x19;

    pub const OPEN_CONNECTION_REQUEST_2: u8 = 0x07;
    pub const OPEN_CONNECTION_REPLY_2: u8 = 0x08;

    pub const CONNECTION_REQUEST: u8 = 0x09;
    pub const CONNECTION_REQUEST_ACCEPTED: u8 = 0x10;
    pub const NEW_INCOMING_CONNECTION: u8 = 0x13;

    pub const CONNECTED_PING: u8 = 0x00;
    pub const CONNECTED_PONG: u8 = 0x03;
    pub const DISCONNECTION_NOTIFICATION: u8 = 0x15;

    pub const FRAME_SET_PACKET_MIN: u8 = 0x80;
    pub const FRAME_SET_PACKET_MAX: u8 = 0x8d;
    pub const NACK: u8 = 0xa0;
    pub const ACK: u8 = 0xc0;
}

pub trait Packet: Sized + Send + Sync + std::fmt::Debug {
    fn id() -> u8;
    fn encode(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()>;
    fn decode(reader: &mut impl BinaryReader) -> BinaryResult<Self>;

    fn serialize_boxed(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_u8(Self::id())?;
        self.encode(writer)
    }
}

macro_rules! implement_packet {
    ($struct_name:ident, $packet_id:expr) => {
        impl $crate::packet::Packet for $struct_name {
            fn id() -> u8 {
                $packet_id
            }

            #[inline]
            fn encode(&self, writer: &mut impl binary::BinaryWriter) -> binary::BinaryResult<()> {
                self.encode_payload(writer)
            }

            #[inline]
            fn decode(reader: &mut impl binary::BinaryReader) -> binary::BinaryResult<Self> {
                Self::decode_payload(reader)
            }
        }
    };
}

pub(crate) use implement_packet; // Hacer macro visible dentro del crate