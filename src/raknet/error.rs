
// src/raknet/error.rs
use crate::utils::binary::BinaryError;
use std::io;
use thiserror::Error;

/// Errors that can occur within the RakNet protocol implementation.
#[derive(Error, Debug)]
pub enum RakNetError {
    /// An I/O error occurred, likely related to the underlying UDP socket.
    #[error("Network I/O error: {0}")]
    Io(#[from] io::Error),

    /// An error occurred during binary serialization or deserialization.
    #[error("Binary handling error: {0}")]
    Binary(#[from] BinaryError),

    /// An invalid packet ID was encountered for the current state or context.
    #[error("Invalid packet ID: {0:#04x}")]
    InvalidPacketId(u8),

    /// Received an unexpected packet for the current session state.
    #[error("Unexpected packet in state {state:?}: {packet_id:#04x}")]
    UnexpectedPacket {
        state: super::session::SessionState,
        packet_id: u8,
    },

    /// Failed to parse or interpret packet data.
    #[error("Packet decode error: {0}")]
    PacketDecode(String),

    /// The maximum transfer unit (MTU) size is invalid or inconsistent.
    #[error("Invalid MTU size: {0}")]
    InvalidMtu(usize),

    /// The client provided an incompatible RakNet protocol version.
    #[error("Incompatible protocol version: client={client}, server={server}")]
    IncompatibleProtocolVersion { client: u8, server: u8 },

    /// Could not find an active session for the given address.
    #[error("Session not found for address: {0}")]
    SessionNotFound(std::net::SocketAddr),

    /// Session timed out due to inactivity.
    #[error("Session timed out")]
    SessionTimeout,

    /// The client disconnected gracefully.
    #[error("Client disconnected gracefully")]
    Disconnected,

    /// The provided advertisement data is too large.
    #[error("Advertisement data too large: {0} bytes")]
    AdvertisementTooLarge(usize),

    /// Received a split packet with invalid parameters.
    #[error("Invalid split packet: {0}")]
    InvalidSplitPacket(String),

    /// Received too many split packets concurrently.
    #[error("Exceeded maximum concurrent split packets")]
    TooManySplitPackets,

    /// An internal error occurred, possibly due to mutex poisoning or unexpected state.
    #[error("Internal RakNet error: {0}")]
    InternalError(String),
}

/// Result type alias for RakNet operations.
pub type Result<T> = std::result::Result<T, RakNetError>;
