
// src/raknet/mod.rs
//! # Amethyst RakNet Implementation
//!
//! Handles the RakNet protocol logic, including connection management,
//! packet reliability, ordering, sequencing, and splitting.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

// --- Submodules ---
pub mod error;
pub mod protocol;
pub mod reliability;
pub mod server_config;
pub mod session;

// --- Re-exports ---
pub use error::RakNetError;
pub use server_config::RakNetServerConfig;
pub use session::RakNetSession;

// --- Constants ---
const MAX_UDP_PACKET_SIZE: usize = 1492; // A common safe MTU, could be configured
const SESSION_CLEANUP_INTERVAL: Duration = Duration::from_secs(5);
const SESSION_TIMEOUT_DURATION: Duration = Duration::from_secs(30); // Consider making this configurable

// --- Server Structure ---

/// High-performance RakNet server implementation for Amethyst.
pub struct RakNetServer {
    socket: Arc<UdpSocket>,
    config: RakNetServerConfig,
    sessions: Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<RakNetSession>>>>>,
    server_guid: u64, // RakNet server unique ID
}

impl RakNetServer {
    /// Binds the RakNet server to the specified address.
    ///
    /// # Arguments
    ///
    /// * `bind_addr` - The socket address to bind the server to.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `RakNetServer` instance or a `RakNetError`.
    pub async fn bind(bind_addr: SocketAddr) -> Result<Self, RakNetError> {
        let socket = UdpSocket::bind(bind_addr).await?;
        let socket = Arc::new(socket);
        info!("RakNet server bound to {}", bind_addr);

        let server_guid = rand::random::<u64>(); // Generate a random server GUID
        info!("Server GUID: {}", server_guid);

        Ok(Self {
            socket,
            config: RakNetServerConfig::default(), // Use default config for now
            sessions: Arc::new(Mutex::new(HashMap::new())),
            server_guid,
        })
    }

    /// Binds the RakNet server with a specific configuration.
    pub async fn bind_with_config(
        bind_addr: SocketAddr,
        config: RakNetServerConfig,
    ) -> Result<Self, RakNetError> {
        let socket = UdpSocket::bind(bind_addr).await?;
        let socket = Arc::new(socket);
        info!("RakNet server bound to {} with custom config", bind_addr);

        let server_guid = config.server_guid.unwrap_or_else(rand::random::<u64>);
        info!("Server GUID: {}", server_guid);

        Ok(Self {
            socket,
            config,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            server_guid,
        })
    }

    /// Runs the main server loop, handling incoming packets and managing sessions.
    /// This function runs indefinitely until an error occurs or the server is stopped.
    pub async fn run(&self) -> Result<(), RakNetError> {
        let mut buf = [0u8; MAX_UDP_PACKET_SIZE]; // Reusable buffer for receiving packets
        let mut last_cleanup = Instant::now();

        loop {
            // Use tokio::select! for potential future additions like shutdown signals
            tokio::select! {
                recv_result = self.socket.recv_from(&mut buf) => {
                    match recv_result {
                        Ok((len, src_addr)) => {
                            let data = &buf[..len];
                            trace!("Received {} bytes from {}", len, src_addr);
                            self.handle_incoming_packet(data, src_addr).await;
                        }
                        Err(e) => {
                            error!("Failed to receive packet: {}", e);
                            // Consider whether specific errors should stop the server
                            // For now, log and continue might be appropriate for UDP
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    // Periodic tasks - run approx every 10ms (RakNet Tick)
                    self.tick_sessions().await;
                    // Run cleanup less frequently
                    if last_cleanup.elapsed() >= SESSION_CLEANUP_INTERVAL {
                        self.cleanup_sessions().await;
                        last_cleanup = Instant::now();
                    }
                }
                // Add signal handling here for graceful shutdown if needed
                // Example:
                // _ = tokio::signal::ctrl_c() => {
                //     info!("Ctrl-C received, shutting down.");
                //     break;
                // }
            }
        }
        // Ok(()) // This part is unreachable in the current loop structure, adjust if adding shutdown
    }

    /// Handles a single incoming UDP packet.
    async fn handle_incoming_packet(&self, data: &[u8], src_addr: SocketAddr) {
        if data.is_empty() {
            trace!("Ignoring empty packet from {}", src_addr);
            return; // Ignore empty packets
        }

        let packet_id = data[0];
        let sessions = self.sessions.lock().await;
        let session_lock = sessions.get(&src_addr).cloned();
        drop(sessions); // Release lock before potentially long-running session handling

        if let Some(session_arc) = session_lock {
            // Packet from a known address, pass to the existing session
            let mut session = session_arc.lock().await;
            session.handle_incoming(data, Instant::now()).await;
        } else {
            // Packet from an unknown address, handle as unconnected
            match packet_id {
                protocol::UNCONNECTED_PING => {
                    protocol::handle_unconnected_ping(
                        self.socket.clone(),
                        src_addr,
                        data,
                        self.server_guid,
                        &self.config,
                    )
                        .await
                }
                protocol::OPEN_CONNECTION_REQUEST_1 => {
                    protocol::handle_open_connection_request_1(
                        self.socket.clone(),
                        src_addr,
                        data,
                        self.server_guid,
                        &self.config,
                    )
                        .await
                }
                protocol::OPEN_CONNECTION_REQUEST_2 => {
                    protocol::handle_open_connection_request_2(
                        self.socket.clone(),
                        self.sessions.clone(),
                        src_addr,
                        data,
                        self.server_guid,
                        &self.config,
                    )
                        .await
                }
                // Ignore other packets from unknown addresses, could be old or invalid
                _ if (packet_id & protocol::datagram::FLAG_VALID) != 0 => {
                    trace!(
                        "Ignoring valid but unknown datagram (ID: {:#04x}) from {}",
                        packet_id,
                        src_addr
                    );
                }
                _ => {
                    trace!(
                        "Ignoring unknown unconnected packet (ID: {:#04x}) from {}",
                        packet_id,
                        src_addr
                    );
                }
            }
        }
    }

    /// Performs periodic tasks for all active sessions.
    async fn tick_sessions(&self) {
        let sessions = self.sessions.lock().await;
        let now = Instant::now();

        // Avoid holding the sessions lock while ticking individual sessions
        let session_pairs: Vec<_> = sessions
            .iter()
            .map(|(addr, session_arc)| (*addr, session_arc.clone()))
            .collect();
        drop(sessions);

        for (addr, session_arc) in session_pairs {
            let mut session = session_arc.lock().await;
            trace!("Ticking session for {}", addr);
            if let Err(e) = session.tick(now).await {
                warn!("Error ticking session {}: {}", addr, e);
                // Optionally remove the session here if tick indicates a critical error
            }
        }
    }

    /// Removes timed-out or disconnected sessions.
    async fn cleanup_sessions(&self) {
        let mut sessions = self.sessions.lock().await;
        let now = Instant::now();
        let initial_count = sessions.len();

        sessions.retain(|addr, session_arc| {
            let session = session_arc.blocking_lock(); // Use blocking lock as we already hold the outer lock
            let should_keep = !session.is_timed_out(now, SESSION_TIMEOUT_DURATION)
                && !session.is_disconnected();
            if !should_keep {
                debug!("Cleaning up session for {}", addr);
                // Perform any final cleanup if necessary before dropping
                // Example: session.notify_disconnect().await; (careful with blocking)
            }
            should_keep
        });

        let removed_count = initial_count - sessions.len();
        if removed_count > 0 {
            debug!("Cleaned up {} sessions", removed_count);
        }
    }
}