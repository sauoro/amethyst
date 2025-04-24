use std::net::SocketAddr;
use rand::random;
use tokio::net::UdpSocket;
use crate::RaknetError;

pub struct RaknetListener {
    pub motd: String,
    pub guid: u64
}

impl RaknetListener {
    pub async fn bind(socket_addr: &SocketAddr) -> Result<Self, RaknetError> {
        let socket = match UdpSocket::bind(socket_addr).await {
            Ok(s) => s,
            Err(_) => return Err(RaknetError::BindAddressError(socket_addr.to_string())),
        };
        
        let server_guid = random::<u64>();
        
        let listener = Self {
            motd: String::new(),
            guid: server_guid
        };
        
        Ok(listener)
    }
}