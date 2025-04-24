pub mod utils;
pub mod raknet;

use raknet::protocol::packet::{
    Packet, PacketId, UnconnectedPing, UnconnectedPong,
};
use utils::{BinaryReader, BinaryWriter};
use bytes::{Bytes, BytesMut};
use rand::Rng;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tracing::{error, info, warn, Level};
use tracing_subscriber;

const SERVER_MOTD: &str = "Amethyst";
const MINECRAFT_VERSION: &str = "1.21.73";
const PROTOCOL_VERSION: u32 = 786;
const MAX_PLAYERS: u32 = 20;
const CURRENT_PLAYERS: u32 = 0;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let bind_addr: SocketAddr = "0.0.0.0:19132".parse()?;
    let socket = UdpSocket::bind(bind_addr).await?;
    info!("Server listening on {}", bind_addr);

    let server_guid: u64 = rand::thread_rng().random();
    info!("Server GUID: {}", server_guid);

    let mut recv_buf = BytesMut::with_capacity(2048);

    loop {
        recv_buf.resize(recv_buf.capacity(), 0);

        let (len, remote_addr) = match socket.recv_from(&mut recv_buf).await {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to receive packet: {}", e);
                continue;
            }
        };

        let mut packet_data = Bytes::copy_from_slice(&recv_buf[..len]);

        let packet_id = match packet_data.get(0) {
            Some(&id) => id,
            None => {
                warn!("Received empty packet from {}", remote_addr);
                continue;
            }
        };

        if packet_id == PacketId::UNCONNECTED_PING {
            match UnconnectedPing::deserialize(&mut packet_data) {
                Ok(ping_packet) => {
                    info!(
                        "Received UnconnectedPing from {} (Timestamp: {})",
                        remote_addr, ping_packet.client_timestamp
                    );

                    let motd_string = format!(
                        "MCPE;{};{};{};{};{};{};{};Survival;1;{};{};",
                        SERVER_MOTD,
                        PROTOCOL_VERSION,
                        MINECRAFT_VERSION,
                        CURRENT_PLAYERS,
                        MAX_PLAYERS,
                        server_guid, // Server GUID (must match pong)
                        "Amethyst",     // Default world name
                        bind_addr.port(), // IPv4 Port
                        bind_addr.port()  // IPv6 Port (can be same)
                    );

                    let pong_packet = UnconnectedPong {
                        server_timestamp: ping_packet.client_timestamp,
                        server_guid,
                        server_name: motd_string,
                    };

                    let mut send_buf = BytesMut::with_capacity(512); // Adjust capacity if needed

                    match pong_packet.serialize(&mut send_buf) {
                        Ok(_) => {
                            if let Err(e) = socket.send_to(&send_buf, remote_addr).await {
                                error!("Failed to send UnconnectedPong to {}: {}", remote_addr, e);
                            } else {
                            }
                        }
                        Err(e) => {
                            error!("Failed to serialize UnconnectedPong: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to decode UnconnectedPing from {}: {}",
                        remote_addr, e
                    );
                }
            }
        } else {
        }
    }
}