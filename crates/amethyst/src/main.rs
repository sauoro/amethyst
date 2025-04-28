use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use bytes::Bytes;
use log::{debug, error, info, logger, trace, warn, Level, Log};
use tokio::net::UdpSocket;
use tokio::time::{Instant, Duration};
use amethyst_binary::io::{BinaryReader, BinaryWriter};
use amethyst_binary::traits::{Readable, Writable};
use amethyst_log::AmethystLogger;
use rakethyst::protocol::{UnconnectedPing, UnconnectedPong, UNCONNECTED_PING, UNCONNECTED_PONG};
use crate::config::Config;
use hex;

pub mod config;

const SERVER_GUID: u64 = 12345678909876543212;
const MINECRAFT_VERSION: &str = "1.20.80";
const PROTOCOL_VERSION: u32 = 662;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = AmethystLogger::init(Level::Trace, 1024) {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }

    let start_time = Instant::now();

    let config: Arc<Config> = match config::handle() {
        Ok(config) => {
            info!("Configuration loaded successfully.");
            logger().flush();
            Arc::new(config)
        },
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            tokio::time::sleep(Duration::from_secs(5)).await;
            return Err(e.into());
        }
    };

    let network_config = &config.network;
    let bind_addr_str = &network_config.address;
    
    let socket: Arc<UdpSocket> = match UdpSocket::bind(bind_addr_str).await {
        Ok(udp_socket) => {
            info!("Listening on {}", bind_addr_str);
            Arc::new(udp_socket)
        }
        Err(e) => {
            error!("Failed to bind UDP socket to {}: {}", bind_addr_str, e);
            return Err(e.into());
        }
    };

    let elapsed_duration = start_time.elapsed();
    info!("Server startup complete in {:.2}s", elapsed_duration.as_secs_f64());
    logger().flush();

    let mut buf = [0u8; 2048];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src_addr)) => {
                let hex_string: String = buf[..len]
                    .iter()
                    .map(|byte| format!("x{:02X}\\", byte))
                    .collect::<Vec<String>>() 
                    .join("");
                trace!("Raw hex from {}: {}", src_addr, hex_string);
                trace!("Received {} bytes from {}", len, src_addr);
                logger().flush();

                let packet_data = Bytes::copy_from_slice(&buf[..len]);

                if packet_data.is_empty() {
                    warn!("Received empty packet from {}", src_addr);
                    logger().flush();
                    continue;
                }

                let packet_id = packet_data[0];

                let socket_clone = Arc::clone(&socket);
                let config_clone = Arc::clone(&config);

                tokio::spawn(handle_packet(
                    socket_clone,
                    packet_data,
                    packet_id,
                    src_addr,
                    config_clone,
                ));
                
            }
            Err(e) => {
                error!("Error receiving UDP packet: {}", e);
            }
        }
    }
    
    info!("Shutting down server");
    logger().flush();
}
async fn handle_packet(
    socket: Arc<UdpSocket>,
    packet_data: Bytes,
    packet_id: u8,
    src_addr: SocketAddr,
    config: Arc<Config>,
) {
    trace!("Handling packet ID {:#04x} from {} ({} bytes)", packet_id, src_addr, packet_data.len());
    logger().flush();
    
    let mut reader = BinaryReader::new(packet_data);

    if reader.read_u8().is_err() {
        error!("Failed to read packet ID from reader despite having data (from {})", src_addr);
        logger().flush();
        return;
    }
    
    match packet_id {
        UNCONNECTED_PING => {
            debug!("Received UNCONNECTED_PING from {}", src_addr);
            logger().flush();
            match UnconnectedPing::read(&mut reader) {
                Ok(ping_packet) => {
                    trace!("Parsed UnconnectedPing: {:?}", ping_packet);
                    logger().flush();
                    
                    let serv_conf = &config.server;

                    let local_addr = match socket.local_addr() {
                        Ok(addr) => addr,
                        Err(e) => {
                            error!("Failed to get local socket address for MOTD: {}", e);
                            return;
                        }
                    };

                    let port_str = local_addr.port().to_string();
                    
                    let ipv4_port_str = match local_addr {
                        SocketAddr::V4(_) => port_str.clone(),
                        SocketAddr::V6(_) => String::from("19132"),
                    };
                    let ipv6_port_str = match local_addr {
                        SocketAddr::V6(_) => port_str.clone(),
                        SocketAddr::V4(_) => String::from("19133"),
                    };

                    let motd = format!(
                        "MCPE;{};{};{};{};{};{};{};{};{};{};{};",
                        serv_conf.name,                   // From config
                        PROTOCOL_VERSION,                 // Constant
                        MINECRAFT_VERSION,                // Constant
                        0,                                // Current players (needs state tracking)
                        serv_conf.max_players,            // From config
                        SERVER_GUID,                      // Constant
                        "Amethyst Default World",         // World Name (Consider making configurable)
                        "Survival",                       // Gamemode (Consider making configurable)
                        1,                                // Nintendo Switch Limited? (Usually 1)
                        &ipv4_port_str,                   // Use actual/derived IPv4 port
                        &ipv6_port_str                    // Use actual/derived IPv6 port
                    );

                    let pong_packet = UnconnectedPong {
                        time: ping_packet.time,
                        server_guid: SERVER_GUID,
                        motd,
                    };

                    let mut writer = BinaryWriter::new();
                    
                    if writer.write_u8(UNCONNECTED_PONG).is_ok() &&
                        pong_packet.write(&mut writer).is_ok()
                    {
                        let response_bytes = writer.freeze();
                        
                        match socket.send_to(response_bytes.as_ref(), src_addr).await {
                            Ok(sent_len) => {
                                debug!("Sent UNCONNECTED_PONG ({} bytes) to {}", sent_len, src_addr);
                                logger().flush();
                            }
                            Err(e) => {
                                error!("Failed to send UNCONNECTED_PONG to {}: {}", src_addr, e);
                                logger().flush();
                            }
                        }
                    } else {
                        error!("Failed to serialize UNCONNECTED_PONG");
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
        _ => {
            if packet_id < 0x80 {
                debug!("Received unhandled offline packet ID {:#04x} from {}", packet_id, src_addr);
                logger().flush();
            } else {
                trace!("Received potential data packet ID {:#04x} from {} (connection not implemented)", packet_id, src_addr);
                logger().flush();
            }
        }
    }
}