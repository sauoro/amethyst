// src/raknet/protocol/mod.rs
//! # RakNet Packet Definitions and Handling
//!
//! Contains structures and functions for specific RakNet packets.

use crate::utils::{binary::*, RakNetError, RakNetServerConfig};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{debug, trace, warn};
use std::collections::HashMap;

// --- Packet IDs ---
pub const UNCONNECTED_PING: u8 = 0x01;
pub const UNCONNECTED_PONG: u8 = 0x1c;
pub const OPEN_CONNECTION_REQUEST_1: u8 = 0x05;
pub const OPEN_CONNECTION_REPLY_1: u8 = 0x06;
pub const OPEN_CONNECTION_REQUEST_2: u8 = 0x07;
pub const OPEN_CONNECTION_REPLY_2: u8 = 0x08;
pub const INCOMPATIBLE_PROTOCOL_VERSION: u8 = 0x19;
pub const CONNECTION_REQUEST: u8 = 0x09;
pub const CONNECTION_REQUEST_ACCEPTED: u8 = 0x10;
pub const NEW_INCOMING_CONNECTION: u8 = 0x13;
pub const DISCONNECT_NOTIFICATION: u8 = 0x15;
pub const CONNECTED_PING: u8 = 0x00;
pub const CONNECTED_PONG: u8 = 0x03;

pub const OFFLINE_MESSAGE_DATA_ID: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

// --- Submodules for Packet Structures ---
pub mod datagram;
pub mod ack;
pub mod offline;

// Re-export important types
pub use datagram::{Datagram, EncapsulatedPacket};
pub use ack::{AckNack, AckNackRecord, ACK, NACK};
pub use offline::*;


// --- Handler Functions ---

/// Handles an Unconnected Ping packet.
pub(super) async fn handle_unconnected_ping(
    socket: Arc<UdpSocket>,
    src_addr: SocketAddr,
    data: &[u8],
    server_guid: u64,
    config: &RakNetServerConfig,
) {
    trace!("Handling Unconnected Ping from {}", src_addr);
    let mut reader = Bytes::copy_from_slice(data);
    if reader.remaining() < 1 + 8 + 16 {
        // Packet ID (1) + Ping ID (8) + Magic (16)
        warn!("Received malformed Unconnected Ping from {}", src_addr);
        return;
    }
    reader.advance(1); // Skip packet ID
    let ping_id = reader.read_i64_be().unwrap_or(0); // Default to 0 on error? Maybe warn?

    let mut magic = [0u8; 16];
    reader.copy_to_slice(&mut magic);
    if magic != OFFLINE_MESSAGE_DATA_ID {
        warn!("Received Unconnected Ping with invalid magic from {}", src_addr);
        return;
    }
    // Client GUID is ignored for ping

    let mut writer = BytesMut::new();
    writer.write_u8(UNCONNECTED_PONG).unwrap();
    writer.write_i64_be(ping_id).unwrap();
    writer.write_i64_be(server_guid as i64).unwrap(); // Write server GUID (BE)
    writer.write_bytes(&OFFLINE_MESSAGE_DATA_ID).unwrap();

    let advertisement_bytes = config.advertisement.as_bytes();
    let ad_len: u16 = advertisement_bytes
        .len()
        .try_into()
        .unwrap_or(u16::MAX); // Should we error if too large?
    if ad_len == u16::MAX && !advertisement_bytes.is_empty() {
        warn!("Advertisement string too long, exceeding 65535 bytes.");
        // Decide: send truncated? send nothing? error? Sending nothing for now.
        writer.write_u16_be(0).unwrap();
    } else {
        writer.write_u16_be(ad_len).unwrap(); // Length of advertisement (BE)
        writer.write_bytes(advertisement_bytes).unwrap();
    }


    if let Err(e) = socket.send_to(&writer, src_addr).await {
        error!("Failed to send Unconnected Pong to {}: {}", src_addr, e);
    } else {
        trace!("Sent Unconnected Pong to {}", src_addr);
    }
}

/// Handles an Open Connection Request 1 packet.
pub(super) async fn handle_open_connection_request_1(
    socket: Arc<UdpSocket>,
    src_addr: SocketAddr,
    data: &[u8],
    server_guid: u64,
    config: &RakNetServerConfig,
) {
    trace!("Handling Open Connection Request 1 from {}", src_addr);
    let mut reader = Bytes::copy_from_slice(data);
    if reader.remaining() < 1 + 16 + 1 {
        // Packet ID (1) + Magic (16) + Protocol Version (1)
        warn!("Received malformed OCR1 from {}", src_addr);
        return;
    }
    reader.advance(1); // Skip packet ID

    let mut magic = [0u8; 16];
    reader.copy_to_slice(&mut magic);
    if magic != OFFLINE_MESSAGE_DATA_ID {
        warn!("Received OCR1 with invalid magic from {}", src_addr);
        return;
    }

    let client_protocol_version = reader.read_u8().unwrap_or(0);
    let raknet_protocol = config.raknet_protocol_version;

    if client_protocol_version != raknet_protocol {
        warn!(
            "Client {} uses incompatible RakNet protocol version {} (server requires {})",
            src_addr, client_protocol_version, raknet_protocol
        );
        send_incompatible_protocol(socket, src_addr, server_guid, raknet_protocol).await;
        return;
    }

    // The rest of the packet is padding up to the MTU size announced by the client.
    // Calculate the MTU size from the received packet length.
    // TotalLength = 1(ID) + 16(Magic) + 1(Proto) + Padding
    // MTU = TotalLength + 20(IPv4) + 8(UDP) OR + 40(IPv6) + 8(UDP)
    let ip_header_size = if src_addr.is_ipv4() { 20 } else { 40 };
    let calculated_mtu = data.len() + ip_header_size + 8;
    let server_mtu = config.mtu as usize;
    // We use the *minimum* of the client's proposed MTU and our server's MTU.
    // Add 1 to dat.len to be safe from MTU limit on client ?
    let final_mtu = std::cmp::min(calculated_mtu, server_mtu);

    debug!(
        "Negotiated MTU with {}: client={}, server={}, final={}",
        src_addr, calculated_mtu, server_mtu, final_mtu
    );

    let mut writer = BytesMut::new();
    writer.write_u8(OPEN_CONNECTION_REPLY_1).unwrap();
    writer.write_bytes(&OFFLINE_MESSAGE_DATA_ID).unwrap();
    writer.write_i64_be(server_guid as i64).unwrap();
    writer.write_u8(0x00).unwrap(); // Use security (false)
    writer.write_u16_be(final_mtu as u16).unwrap(); // Final MTU (BE)


    if let Err(e) = socket.send_to(&writer, src_addr).await {
        error!(
            "Failed to send Open Connection Reply 1 to {}: {}",
            src_addr, e
        );
    } else {
        trace!("Sent Open Connection Reply 1 to {}", src_addr);
    }
}

/// Handles an Open Connection Request 2 packet.
pub(super) async fn handle_open_connection_request_2(
    socket: Arc<UdpSocket>,
    sessions: Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<crate::RakNetSession>>>>>,
    src_addr: SocketAddr,
    data: &[u8],
    server_guid: u64,
    config: &RakNetServerConfig,
) {
    trace!("Handling Open Connection Request 2 from {}", src_addr);
    let mut reader = Bytes::copy_from_slice(data);
    if reader.remaining() < 1 + 16 { // Packet ID + Magic is the minimum
        warn!("Received malformed OCR2 from {}", src_addr);
        return;
    }
    reader.advance(1); // Skip packet ID

    let mut magic = [0u8; 16];
    reader.copy_to_slice(&mut magic);
    if magic != OFFLINE_MESSAGE_DATA_ID {
        warn!("Received OCR2 with invalid magic from {}", src_addr);
        return;
    }

    // Decode Server Address, MTU Size, Client GUID
    let server_addr = match read_address(&mut reader) {
        Ok(addr) => addr,
        Err(e) => {
            warn!("Failed to decode server address in OCR2 from {}: {}", src_addr, e);
            return;
        }
    };
    let mtu_size = match reader.read_u16_be() {
        Ok(mtu) => mtu as usize,
        Err(e) => {
            warn!("Failed to decode MTU in OCR2 from {}: {}", src_addr, e);
            return;
        }
    };
    let client_guid = match reader.read_i64_be() {
        Ok(guid) => guid,
        Err(e) => {
            warn!("Failed to decode client GUID in OCR2 from {}: {}", src_addr, e);
            return;
        }
    };

    // Validate MTU again, more strictly now as it's client-declared
    if mtu_size < offline::MIN_MTU_SIZE || mtu_size > config.mtu as usize {
        warn!("Client {} sent OCR2 with invalid MTU size: {} (server range: {}-{})", src_addr, mtu_size, offline::MIN_MTU_SIZE, config.mtu);
        // Optionally disconnect or ignore
        return;
    }


    // Send Open Connection Reply 2
    let mut writer = BytesMut::new();
    writer.write_u8(OPEN_CONNECTION_REPLY_2).unwrap();
    writer.write_bytes(&OFFLINE_MESSAGE_DATA_ID).unwrap();
    writer.write_i64_be(server_guid as i64).unwrap();
    write_address(&mut writer, &src_addr).expect("Failed to write client address"); // Send back client's address
    writer.write_u16_be(mtu_size as u16).unwrap(); // Acknowledge the MTU (BE)
    writer.write_u8(0x00).unwrap(); // Use encryption (false)

    if let Err(e) = socket.send_to(&writer, src_addr).await {
        error!("Failed to send Open Connection Reply 2 to {}: {}", src_addr, e);
        return; // Don't create a session if we can't even reply
    }
    trace!("Sent Open Connection Reply 2 to {}", src_addr);


    // --- Create the session ---
    // Lock the sessions map exclusively
    let mut sessions_map = sessions.lock().await;

    if sessions_map.contains_key(&src_addr) {
        warn!("Session for {} already exists, ignoring OCR2.", src_addr);
        // Potentially disconnect the old session? Or just ignore the new request.
        // For now, just ignore.
        return;
    }

    // Check connection limit
    if sessions_map.len() >= config.max_connections {
        warn!("Max connections ({}) reached, rejecting connection from {}", config.max_connections, src_addr);
        // Optionally send a specific disconnect packet (like NO_FREE_INCOMING_CONNECTIONS if defined)
        return;
    }


    let session = Arc::new(Mutex::new(crate::RakNetSession::new(
        src_addr,
        client_guid,
        mtu_size,
        socket.clone(), // Give the session the Arc<UdpSocket>
        server_guid,
    )));

    sessions_map.insert(src_addr, session.clone());
    debug!("Created new session for {} (GUID: {})", src_addr, client_guid);

    // Spawn a task to handle the new incoming connection packet AFTER releasing the lock
    // This is important to avoid deadlocks if the session immediately tries to send something
    // that requires the server's socket (which handle_incoming_packet uses).
    // We need to release the outer sessions lock first.
    drop(sessions_map);

    tokio::spawn(async move {
        let mut s = session.lock().await;
        s.handle_new_incoming_connection().await;
    });

}

/// Sends an Incompatible Protocol Version packet.
async fn send_incompatible_protocol(
    socket: Arc<UdpSocket>,
    src_addr: SocketAddr,
    server_guid: u64,
    server_protocol_version: u8,
) {
    let mut writer = BytesMut::new();
    writer.write_u8(INCOMPATIBLE_PROTOCOL_VERSION).unwrap();
    writer.write_u8(server_protocol_version).unwrap();
    writer.write_bytes(&OFFLINE_MESSAGE_DATA_ID).unwrap();
    writer.write_i64_be(server_guid as i64).unwrap();

    if let Err(e) = socket.send_to(&writer, src_addr).await {
        error!(
            "Failed to send Incompatible Protocol Version to {}: {}",
            src_addr, e
        );
    } else {
        trace!("Sent Incompatible Protocol Version to {}", src_addr);
    }
}


// --- Helper Functions ---

/// Writes a SocketAddr (IPv4 or IPv6) to the buffer in RakNet format.
pub fn write_address(writer: &mut BytesMut, addr: &SocketAddr) -> Result<(), BinaryError> {
    match addr {
        SocketAddr::V4(v4_addr) => {
            writer.put_u8(4); // Address type IPv4
            // Write IP address bytes inverted
            for byte in v4_addr.ip().octets().iter() {
                writer.put_u8(!byte);
            }
            writer.write_u16_be(v4_addr.port())?; // Port (BE)
        }
        SocketAddr::V6(v6_addr) => {
            writer.put_u8(6); // Address type IPv6
            writer.write_u16_le(23)?; // Address family (AF_INET6 on Windows?) - use LE as observed
            writer.write_u16_be(v6_addr.port())?; // Port (BE)
            writer.write_u32_be(v6_addr.flowinfo())?; // Flow info (BE)
            writer.put(&v6_addr.ip().octets()[..]); // IP address bytes
            writer.write_u32_be(v6_addr.scope_id())?; // Scope ID (BE) - seems LE in JS version, use BE per pmmp
        }
    }
    Ok(())
}

/// Reads a SocketAddr (IPv4 or IPv6) from the buffer in RakNet format.
pub fn read_address(reader: &mut Bytes) -> Result<SocketAddr, BinaryError> {
    if reader.remaining() < 1 {
        return Err(BinaryError::UnexpectedEof { needed: 1, remaining: reader.remaining() });
    }
    let addr_type = reader.get_u8();
    match addr_type {
        4 => { // IPv4
            if reader.remaining() < 4 + 2 {
                return Err(BinaryError::UnexpectedEof { needed: 6, remaining: reader.remaining() });
            }
            let mut ip_bytes = [0u8; 4];
            reader.copy_to_slice(&mut ip_bytes);
            // Invert bytes back
            for byte in ip_bytes.iter_mut() {
                *byte = !*byte;
            }
            let ip = std::net::Ipv4Addr::from(ip_bytes);
            let port = reader.read_u16_be()?;
            Ok(SocketAddr::V4(std::net::SocketAddrV4::new(ip, port)))
        }
        6 => { // IPv6
            if reader.remaining() < 2 + 2 + 4 + 16 + 4 {
                return Err(BinaryError::UnexpectedEof { needed: 28, remaining: reader.remaining() });
            }
            let _family = reader.read_u16_le()?; // Read family (LE) - but don't use it?
            let port = reader.read_u16_be()?;     // Read port (BE)
            let flowinfo = reader.read_u32_be()?;  // Read flowinfo (BE)
            let mut ip_bytes = [0u8; 16];
            reader.copy_to_slice(&mut ip_bytes);
            let ip = std::net::Ipv6Addr::from(ip_bytes);
            let scope_id = reader.read_u32_be()?; // Read scope_id (BE) - seems LE in JS, BE in pmmp
            Ok(SocketAddr::V6(std::net::SocketAddrV6::new(ip, port, flowinfo, scope_id)))
        }
        _ => Err(BinaryError::InvalidData(format!(
            "Unknown address type: {}",
            addr_type
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::{BinaryReader, BinaryWritter}; // Use crate::binary for tests within raknet::protocol
    use bytes::{Bytes, BytesMut};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};


    #[test]
    fn test_read_write_address_v4() {
        let addr_v4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 19132);
        let mut writer = BytesMut::new();
        write_address(&mut writer, &addr_v4).unwrap();

        // Expected format: 0x04 | !192 | !168 | !1 | !100 | 19132_BE
        // !192 = 63 (0x3F), !168 = 87 (0x57), !1 = 254 (0xFE), !100 = 155 (0x9B)
        // 19132_BE = 0x4A BC
        let expected_v4 = Bytes::from_static(&[0x04, 63, 87, 254, 155, 0x4A, 0xBC]);
        assert_eq!(writer.freeze(), expected_v4);


        let mut reader_bytes = expected_v4.clone(); // Clone Bytes for reading
        let read_addr = read_address(&mut reader_bytes).unwrap();
        assert_eq!(read_addr, addr_v4);
        assert!(reader_bytes.is_empty());

    }

    #[test]
    fn test_read_write_address_v6() {
        let ip_v6 = Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334);
        let addr_v6 = SocketAddr::new(IpAddr::V6(ip_v6), 19133);
        // Assuming flowinfo = 0, scope_id = 0 for simplicity in testing basic encoding
        let addr_v6_full = std::net::SocketAddrV6::new(ip_v6, 19133, 0, 0);


        let mut writer = BytesMut::new();
        write_address(&mut writer, &SocketAddr::V6(addr_v6_full)).unwrap();

        // Expected format: 0x06 | 23_LE (0x17 0x00) | port_BE (0x4A BD) | flowinfo_BE (0) | ip | scope_id_BE (0)
        let expected_v6 = Bytes::from_static(&[
            0x06, 0x17, 0x00, 0x4A, 0xBD, 0x00, 0x00, 0x00, 0x00, // header + port + flowinfo
            0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34, // ip
            0x00, 0x00, 0x00, 0x00, // scope_id
        ]);
        assert_eq!(writer.freeze(), expected_v6);


        let mut reader_bytes = expected_v6.clone();
        let read_addr = read_address(&mut reader_bytes).unwrap();

        // Check specifically against SocketAddr::V6
        if let SocketAddr::V6(read_v6) = read_addr {
            assert_eq!(read_v6.ip(), &ip_v6);
            assert_eq!(read_v6.port(), 19133);
            assert_eq!(read_v6.flowinfo(), 0); // Expected flowinfo
            assert_eq!(read_v6.scope_id(), 0); // Expected scope_id
        } else {
            panic!("Expected SocketAddr::V6");
        }
        assert!(reader_bytes.is_empty());

    }

    #[test]
    fn test_read_invalid_address_type() {
        let mut reader_bytes = Bytes::from_static(&[0x07, 0x01, 0x02, 0x03]); // Invalid type 7
        let result = read_address(&mut reader_bytes);
        assert!(matches!(result, Err(BinaryError::InvalidData(_))));
    }

    #[test]
    fn test_read_unexpected_eof() {
        // Test IPv4 EOF
        let mut reader_eof_v4 = Bytes::from_static(&[0x04, 0x01, 0x02]); // Too short
        let result_v4 = read_address(&mut reader_eof_v4);
        assert!(matches!(result_v4, Err(BinaryError::UnexpectedEof{..})));

        // Test IPv6 EOF
        let mut reader_eof_v6 = Bytes::from_static(&[0x06, 0x17, 0x00, 0x4A, 0xBD, 0x00, 0x00, 0x00, 0x00, 0x20, 0x01]); // Too short
        let result_v6 = read_address(&mut reader_eof_v6);
        assert!(matches!(result_v6, Err(BinaryError::UnexpectedEof{..})));
    }
}