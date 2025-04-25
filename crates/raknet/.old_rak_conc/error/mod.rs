use std::net::SocketAddr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RaknetError {
    #[error("Packet parsing error: {0}")]
    PacketParseError(String),
    #[error("Incompatible RakNet protocol version: client={client}, server supports={server}")]
    IncompatibleProtocolVersion { client: u8, server: u8 },
    #[error("Invalid packet ID encountered: {0:#04x}")]
    InvalidPacketId(u8),
    #[error("Invalid reliability flags: {0}")]
    InvalidReliabilityFlags(u8),
    #[error("Packet too large for MTU (size={size}, mtu={mtu})")]
    PacketTooLarge { size: usize, mtu: u16 },
    #[error("Fragmented packet error: {0}")]
    FragmentError(String),
    #[error("Binary I/O error: {0}")]
    BinaryError(#[from] binary::BinaryError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Data conversion error: {0}")]
    ConversionError(String),
    #[error("Connection timed out")]
    Timeout,
    #[error("Connection closed by peer")]
    ConnectionClosed,
    #[error("Connection reset by peer")]
    ConnectionReset,
    #[error("Address already in use: {0}")]
    AddrInUse(SocketAddr),
    #[error("Address not available: {0}")]
    AddrNotAvailable(SocketAddr),
    #[error("Not connected")]
    NotConnected,
    #[error("Already connected")]
    AlreadyConnected,
    #[error("Handshake error: {0}")]
    HandshakeError(String),
    #[error("Internal RakNet error: {0}")]
    InternalError(String),
    #[error("Failed to bind socket address: {0}")]
    BindAddressError(String),
}

pub type Result<T> = std::result::Result<T, RaknetError>;