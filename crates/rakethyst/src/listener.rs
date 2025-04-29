use crate::connection::{Connection, ConnectionState};
use crate::protocol;
use crate::protocol::{ConnectionRequest, ConnectionRequestAccepted, OpenConnectionReply1, OpenConnectionReply2, OpenConnectionRequest1, OpenConnectionRequest2, UnconnectedPing, UnconnectedPong, CONNECTION_REQUEST_ACCEPTED, OPEN_CONNECTION_REPLY_2, UNCONNECTED_PONG};
use amethyst_binary::io::{BinaryReader, BinaryWriter};
use amethyst_binary::traits::{Readable, Writable};
use bytes::Bytes;
use dashmap::DashMap;
use log::{debug, error, info, logger, trace, warn};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;

const SERVER_GUID: u64 = 12345678909876543212;
const MINECRAFT_VERSION: &str = "1.20.80";
const PROTOCOL_VERSION: u32 = 662;

pub struct RakNetListener {
    socket: Arc<UdpSocket>,
    server_name: Arc<String>,
    connections: Arc<DashMap<SocketAddr, Connection>>,
}

impl RakNetListener {
    pub async fn bind(addr: &str, server_name: String) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(addr)?;
        info!("RakNet listener bound to {}", addr);
        Ok(Self {
            socket: Arc::new(socket),
            server_name: Arc::new(server_name.into()),
            connections: Arc::new(DashMap::new()),
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = [0u8; 2048];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((len, src_addr)) => {
                    if len == 0 {
                        warn!("Received empty packet from {}", src_addr);
                        continue;
                    }

                    let packet_data = Bytes::copy_from_slice(&buf[..len]);

                    let socket_clone = Arc::clone(&self.socket);
                    let server_name_clone = self.server_name.clone();
                    let connections_clone = Arc::clone(&self.connections);

                    tokio::spawn(handle_packet(
                        socket_clone,
                        packet_data,
                        src_addr,
                        server_name_clone,
                        connections_clone,
                    ));
                }
                Err(e) => {
                    error!("Error receiving UDP packet: {}", e);
                    return Err(e.into());
                }
            }
        }
    }
}

async fn handle_packet(
    socket: Arc<UdpSocket>,
    packet_data: Bytes,
    src_addr: SocketAddr,
    server_name: Arc<String>,
    connections: Arc<DashMap<SocketAddr, Connection>>,
) {
    if packet_data.is_empty() {
        warn!("handle_packet received empty data from {}", src_addr);
        return;
    }

    let packet_id = packet_data[0];
    trace!(
        "Handling packet ID {:#04x} from {} ({} bytes)",
        packet_id,
        src_addr,
        packet_data.len()
    );
    logger().flush();

    let mut reader = BinaryReader::new(packet_data);

    if reader.read_u8().is_err() {
        error!(
            "Failed to advance reader past packet ID from {} (data len: {})",
            src_addr,
            reader.remaining() + 1
        );
        logger().flush();
        return;
    }

    match packet_id {
        protocol::UNCONNECTED_PING => {
            debug!("Received UNCONNECTED_PING from {}", src_addr);
            logger().flush();
            match UnconnectedPing::read(&mut reader) {
                Ok(ping_packet) => {
                    trace!("Parsed UnconnectedPing: {:?}", ping_packet);
                    logger().flush();

                    let local_addr = match socket.local_addr() {
                        Ok(addr) => addr,
                        Err(e) => {
                            error!("Failed to get local socket address for MOTD: {}", e);
                            logger().flush();
                            return;
                        }
                    };
                    let port_str = local_addr.port().to_string();
                    let ipv4_port_str = match local_addr {
                        SocketAddr::V4(v4) if v4.port() == 19132 => port_str.clone(),
                        _ => "19132".to_string(),
                    };
                    let ipv6_port_str = match local_addr {
                        SocketAddr::V6(v6) if v6.port() == 19133 => port_str.clone(),
                        _ => "19133".to_string(),
                    };

                    let motd = format!(
                        "MCPE;{};{};{};{};{};{};{};{};{};{};{};",
                        server_name,
                        PROTOCOL_VERSION,
                        MINECRAFT_VERSION,
                        0,
                        50,
                        SERVER_GUID,
                        "Amethyst World",
                        "Survival",
                        1,
                        &ipv4_port_str,
                        &ipv6_port_str
                    );

                    let pong_packet = UnconnectedPong {
                        time: ping_packet.time,
                        server_guid: SERVER_GUID,
                        motd,
                    };

                    let mut writer = BinaryWriter::new();

                    if writer.write_u8(UNCONNECTED_PONG).is_ok()
                        && pong_packet.write(&mut writer).is_ok()
                    {
                        let response_bytes = writer.freeze();

                        match socket.send_to(response_bytes.as_ref(), src_addr) {
                            Ok(sent_len) => {
                                debug!(
                                    "Sent UNCONNECTED_PONG ({} bytes) to {}",
                                    sent_len, src_addr
                                );
                                logger().flush();
                            }
                            Err(e) => {
                                error!("Failed to send UNCONNECTED_PONG to {}: {}", src_addr, e);
                                logger().flush();
                            }
                        }
                    } else {
                        error!("Failed to serialize UNCONNECTED_PONG for {}", src_addr);
                        logger().flush();
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to parse UNCONNECTED_PING payload from {}: {}",
                        src_addr, e
                    );
                    logger().flush();
                }
            }
        }
        protocol::OPEN_CONNECTION_REQUEST_1 => {
            debug!("Received OPEN_CONNECTION_REQUEST_1 from {}", src_addr);
            logger().flush();
            match OpenConnectionRequest1::read(&mut reader) {
                Ok(request) => {
                    trace!("Parsed OpenConnectionRequest1: {:?}", request);

                    if request.protocol_version != protocol::RAKNET_PROTOCOL_VERSION {
                        warn!(
                            "Client {} sent unsupported RakNet protocol version: {} (expected: {})",
                            src_addr,
                            request.protocol_version,
                            protocol::RAKNET_PROTOCOL_VERSION
                        );
                        return;
                    }

                    let server_mtu: u16 = 1400;

                    let reply = OpenConnectionReply1 {
                        server_guid: SERVER_GUID,
                        use_security: false,
                        mtu_size: server_mtu,
                    };

                    let mut writer = BinaryWriter::new();
                    if writer.write_u8(protocol::OPEN_CONNECTION_REPLY_1).is_ok()
                        && reply.write(&mut writer).is_ok()
                    {
                        let response_bytes = writer.freeze();
                        match socket.send_to(response_bytes.as_ref(), src_addr) {
                            Ok(sent_len) => debug!(
                                "Sent OPEN_CONNECTION_REPLY_1 ({} bytes, MTU: {}) to {}",
                                sent_len, server_mtu, src_addr
                            ),
                            Err(e) => error!(
                                "Failed to send OPEN_CONNECTION_REPLY_1 to {}: {}",
                                src_addr, e
                            ),
                        }
                    } else {
                        error!(
                            "Failed to serialize OPEN_CONNECTION_REPLY_1 for {}",
                            src_addr
                        );
                    }
                }
                Err(e) => warn!(
                    "Failed to parse OPEN_CONNECTION_REQUEST_1 from {}: {}",
                    src_addr, e
                ),
            }
            logger().flush();
        }
        protocol::OPEN_CONNECTION_REQUEST_2 => {
            debug!("Received OPEN_CONNECTION_REQUEST_2 from {}", src_addr);
            match OpenConnectionRequest2::read(&mut reader) {
                Ok(request) => {
                    trace!("Parsed OpenConnectionRequest2: {:?}", request);

                    let server_max_mtu = 1400; // Must match MTU logic from Reply1 handler
                    let final_mtu = request.mtu.min(server_max_mtu).max(400); // Ensure a minimum MTU

                    let reply = OpenConnectionReply2 {
                        server_guid: SERVER_GUID,
                        client_addr: src_addr,
                        mtu: final_mtu,
                        use_encryption: false,
                    };

                    let mut writer = BinaryWriter::new();
                    if writer.write_u8(OPEN_CONNECTION_REPLY_2).is_ok()
                        && reply.write(&mut writer).is_ok()
                    {
                        let response_bytes = writer.freeze();
                        match socket.send_to(response_bytes.as_ref(), src_addr) {
                            Ok(sent_len) => debug!(
                                "Sent OPEN_CONNECTION_REPLY_2 ({} bytes, MTU: {}) to {}",
                                sent_len, final_mtu, src_addr
                            ),
                            Err(e) => error!(
                                "Failed to send OPEN_CONNECTION_REPLY_2 to {}: {}",
                                src_addr, e
                            ),
                        }
                    } else {
                        error!(
                            "Failed to serialize OPEN_CONNECTION_REPLY_2 for {}",
                            src_addr
                        );
                    }
                }
                Err(e) => warn!(
                    "Failed to parse OPEN_CONNECTION_REQUEST_2 from {}: {}",
                    src_addr, e
                ),
            }
            logger().flush();
        }
        protocol::CONNECTION_REQUEST => {
            if let Some(mut conn_entry) = connections.get_mut(&src_addr) {
                if conn_entry.state == ConnectionState::Connected
                    || conn_entry.state == ConnectionState::Connecting
                {
                    debug!(
                        "Received duplicate CONNECTION_REQUEST from already known address {}",
                        src_addr
                    );
                    conn_entry.update_last_packet_time();
                    return;
                }
            }

            match ConnectionRequest::read(&mut reader) {
                Ok(request) => {
                    debug!(
                        "Received CONNECTION_REQUEST from {} (Client GUID: {}, Time: {}, Security: {})",
                        src_addr, request.client_guid, request.time, request.use_security
                    );
                    trace!("Parsed ConnectionRequest: {:?}", request);
                    let agreed_mtu = 1400;

                    let mut new_connection =
                        Connection::new(src_addr, request.client_guid, agreed_mtu);
                    new_connection.state = ConnectionState::Connecting;

                    let system_address = socket.local_addr().unwrap_or_else(|_| {
                        SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0)
                    });

                    let reply = ConnectionRequestAccepted {
                        client_address: src_addr,
                        system_index: 0,
                        internal_ids: [system_address; 20],
                        request_time: request.time,
                        time: crate::utils::cur_time_millis(),
                    };
                    let mut writer = BinaryWriter::new();
                    if writer.write_u8(CONNECTION_REQUEST_ACCEPTED).is_ok()
                        && reply.write(&mut writer).is_ok()
                    {
                        let response_bytes = writer.freeze();
                        match socket.send_to(response_bytes.as_ref(), src_addr) {
                            Ok(sent_len) => {
                                debug!(
                                    "Sent CONNECTION_REQUEST_ACCEPTED ({} bytes) to {}",
                                    sent_len, src_addr
                                );
                                // Insert/update the connection state *after* successfully sending the reply
                                connections.insert(src_addr, new_connection);
                            }
                            Err(e) => error!(
                                "Failed to send CONNECTION_REQUEST_ACCEPTED to {}: {}",
                                src_addr, e
                            ),
                        }
                    } else {
                        error!(
                            "Failed to serialize CONNECTION_REQUEST_ACCEPTED for {}",
                            src_addr
                        );
                    }
                }
                Err(e) => warn!(
                    "Failed to parse CONNECTION_REQUEST from {}: {}",
                    src_addr, e
                ),
            }
            logger().flush();
        }
        0x80..=0x8F => {
            trace!("Received potential data frame {:#04x} from {}", packet_id, src_addr);
            if let Some(mut connection_entry) = connections.get_mut(&src_addr) {
                let connection = connection_entry.value_mut();
                connection.update_last_packet_time();

                if connection.state == ConnectionState::Connecting {
                    debug!("Connection from {} promoted to Connected state.", src_addr);
                    connection.state = ConnectionState::Connected;
                }

                if connection.state == ConnectionState::Connected {
                    warn!("Received data frame {:#04x}, but reliability layer not implemented yet for {}. Dropping.", packet_id, src_addr);
                } else {
                    warn!("Received data frame {:#04x} from {} in unexpected state {:?}. Dropping.", packet_id, src_addr, connection.state);
                }
            } else {
                warn!("Received data frame {:#04x} from unknown address {}. Dropping.", packet_id, src_addr);
            }
            logger().flush();
        }
        _ => {
            if packet_id < 0x80 {
                debug!(
                    "Received unhandled offline RakNet packet ID {:#04x} from {}",
                    packet_id, src_addr
                );
                logger().flush();
            } else {
                trace!(
                    "Received potential data packet ID {:#04x} from {} (no connection)",
                    packet_id, src_addr
                );
                logger().flush();
            }
        }
    }
}
