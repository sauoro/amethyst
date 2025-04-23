// src/raknet/protocol/offline.rs
//! Structures and handlers specific to offline/unconnected RakNet packets.

use crate::utils::binary::*;
use crate::raknet::protocol::{write_address, OFFLINE_MESSAGE_DATA_ID, RakNetError};

// --- Constants ---
pub const MIN_MTU_SIZE: usize = 400; // Minimum reliable MTU size

// --- Packet Structures ---

#[derive(Debug, Clone)]
pub struct UnconnectedPing {
    pub ping_id: i64, // Time BE
    // Magic is implicit
    pub client_guid: i64, // Time BE
}

#[derive(Debug, Clone)]
pub struct UnconnectedPong {
    pub ping_id: i64,         // Time BE
    pub server_guid: u64,     // u64 GUID BE
    // Magic is implicit
    pub advertisement: String, // u16 len BE + String
}

#[derive(Debug, Clone)]
pub struct OpenConnectionRequest1 {
    // Magic is implicit
    pub protocol_version: u8,
    // MTU size is derived from padding
    // This struct doesn't explicitly store padding or MTU
}

#[derive(Debug, Clone)]
pub struct OpenConnectionReply1 {
    // Magic is implicit
    pub server_guid: u64, // u64 GUID BE
    pub use_security: bool,
    pub mtu_size: u16, // u16 MTU BE
}

#[derive(Debug, Clone)]
pub struct OpenConnectionRequest2 {
    // Magic is implicit
    // Potentially security challenge data here if security is enabled
    pub server_address: std::net::SocketAddr,
    pub mtu_size: u16,    // u16 MTU BE
    pub client_guid: i64, // i64 GUID BE
}

#[derive(Debug, Clone)]
pub struct OpenConnectionReply2 {
    // Magic is implicit
    pub server_guid: u64,                    // u64 GUID BE
    pub client_address: std::net::SocketAddr, // Client's perceived address
    pub mtu_size: u16,                       // u16 MTU BE
    pub use_encryption: bool,
}

#[derive(Debug, Clone)]
pub struct IncompatibleProtocolVersion {
    pub server_protocol_version: u8,
    // Magic is implicit
    pub server_guid: u64, // u64 GUID BE
}