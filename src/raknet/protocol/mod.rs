pub mod ack;
pub mod frame;
pub mod packet;
pub mod reliability;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum RaknetError {
    #[error("Packet parsing error: {0}")]
    PacketParseError(String),
    #[error("Incompatible RakNet protocol version: client={client}, server supports={server:?}")]
    IncompatibleProtocolVersion { client: u8, server: &'static [u8] },
    #[error("Invalid packet ID encountered: {0}")]
    InvalidPacketId(u8),
    #[error("Invalid reliability flags: {0}")]
    InvalidReliabilityFlags(u8),
    #[error("Packet too large for MTU")]
    PacketTooLarge,
    #[error("Fragmented packet error: {0}")]
    FragmentError(String),
    #[error("Binary I/O error: {0}")]
    BinaryError(#[from] crate::utils::binary::BinaryError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Data conversion error: {0}")]
    ConversionError(String),
}
pub type Result<T> = std::result::Result<T, RaknetError>;
