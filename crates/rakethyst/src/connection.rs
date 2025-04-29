use std::net::SocketAddr;
use tokio::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Handshaking,
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub address: SocketAddr,
    pub client_guid: u64,
    pub mtu: u16,
    pub state: ConnectionState,
    pub last_packet_time: Instant,
}

impl Connection {
    pub fn new(address: SocketAddr, client_guid: u64, mtu: u16) -> Self {
        Connection {
            address,
            client_guid,
            mtu,
            state: ConnectionState::Handshaking,
            last_packet_time: Instant::now(),
        }
    }

    pub fn update_last_packet_time(&mut self) {
        self.last_packet_time = Instant::now();
    }
}