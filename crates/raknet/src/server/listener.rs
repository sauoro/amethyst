use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use rand::random;
use tokio::net::UdpSocket;
use tokio::sync::Notify;
use crate::{RaknetError};

pub struct RaknetListener {
    pub motd: String,
    pub guid: u64,
    pub version_map: Arc<Mutex<HashMap<String, u8>>>,
    addr: Option<Arc<UdpSocket>>,
    listened: bool,
    // connections: Arc<Mutex<HashMap<SocketAddr, Session>>>,
    // connection_receiver: Receiver<Connection>,
    // connection_sender: Sender<Connection>,
    closed_notify: Arc<Notify>,
}

impl RaknetListener {
    pub async fn bind(socket_addr: &SocketAddr) -> Result<Self, RaknetError> {
        let socket = match UdpSocket::bind(socket_addr).await {
            Ok(s) => s,
            Err(_) => return Err(RaknetError::BindAddressError(socket_addr.to_string())),
        };

        let server_guid = random::<u64>();

        //let (conn_sender, conn_receiver) = channel::<Connection>();

        let listener = Self {
            motd: String::new(),
            guid: server_guid,
            version_map: Arc::new(Mutex::new(HashMap::new())),
            addr: Some(Arc::new(socket)),
            listened: false,
            //connection_sender: conn_sender,
            //connection_receiver: conn_receiver,
            closed_notify: Arc::new(Notify::new()),
        };

        Ok(listener)
    }
}