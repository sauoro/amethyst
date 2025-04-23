// src/raknet/server_config.rs
use crate::raknet::protocol::OFFLINE_MESSAGE_DATA_ID;
use bytes::Bytes;

/// Configuration options for the RakNetServer.
#[derive(Clone, Debug)]
pub struct RakNetServerConfig {
    /// The server's unique 64-bit GUID. If None, a random one will be generated.
    pub server_guid: Option<u64>,
    /// The maximum number of incoming connections allowed.
    pub max_connections: usize,
    /// The RakNet protocol version the server will advertise and accept.
    pub raknet_protocol_version: u8,
    /// The server advertisement data sent in Unconnected Pong packets.
    pub advertisement: Bytes,
    /// The magic bytes used for unconnected messages. Should usually be the RakNet default.
    pub unconnected_magic: [u8; 16],
    /// The Maximum Transmission Unit (MTU) size supported by the server.
    /// Packets larger than this (after RakNet overhead) will be split.
    pub mtu: u16,
    // Add more options as needed:
    // pub session_timeout: Duration,
    // pub enable_security: bool, // Maybe later? RakNet security is complex.
}

impl Default for RakNetServerConfig {
    fn default() -> Self {
        Self {
            server_guid: None,
            max_connections: 1000, // Default max connections
            raknet_protocol_version: 11, // Current Minecraft Bedrock protocol version
            // Default advertisement (can be customized) - Follows MCPE format
            advertisement: Bytes::from(
                "MCPE;Amethyst Server;11;1.20.0;0;10;0;Amethyst RakNet;Survival"
            ),
            unconnected_magic: OFFLINE_MESSAGE_DATA_ID,
            mtu: 1400, // A common MTU value
        }
    }
}