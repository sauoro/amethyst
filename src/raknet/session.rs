// src/raknet/session.rs
//! Handles the state and logic for a single connected RakNet client session.

use crate::utils::binary::BinaryWritter;
use crate::raknet::protocol::{self, Datagram, EncapsulatedPacket, Reliability, RakNetError};
use crate::raknet::reliability::{ReceiveWindow, SendWindow, SplitPacketHandler};
use bytes::BytesMut;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tracing::{debug, trace, warn};

/// Represents the current state of a RakNet session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Initial state after OCR2, waiting for Connection Request.
    Connecting,
    /// Received Connection Request, waiting for New Incoming Connection (from client).
    Handshaking,
    /// Connection fully established.
    Connected,
    /// Disconnect initiated, waiting for final packets/acks.
    Disconnecting,
    /// Session is closed.
    Disconnected,
}

/// Represents a connected RakNet session.
pub struct RakNetSession {
    /// Remote address of the client.
    address: SocketAddr,
    /// GUID of the client.
    client_guid: i64,
    /// Maximum Transmission Unit size for this session.
    mtu: usize,
    /// Current state of the session.
    state: SessionState,
    /// Timestamp of the last received packet.
    last_activity: Instant,
    /// Shared UDP socket for sending packets.
    socket: Arc<UdpSocket>,
    /// Server GUID (used in some packets).
    server_guid: u64,
    /// Handler for reliable sending, ACKs, NACKs, and retransmission.
    send_window: SendWindow,
    /// Handler for reliable receiving, managing sequence numbers, and generating ACKs/NACKs.
    receive_window: ReceiveWindow,
    /// Handler for reassembling split packets.
    split_handler: SplitPacketHandler,
}

impl RakNetSession {
    /// Creates a new RakNet session.
    pub fn new(
        address: SocketAddr,
        client_guid: i64,
        mtu: usize,
        socket: Arc<UdpSocket>,
        server_guid: u64,
    ) -> Self {
        debug!(
            "Creating new session for {} (GUID: {}, MTU: {})",
            address, client_guid, mtu
        );
        Self {
            address,
            client_guid,
            mtu,
            state: SessionState::Connecting, // Starts after OCREP2, expecting Connection Request
            last_activity: Instant::now(),
            socket,
            server_guid,
            send_window: SendWindow::new(mtu),
            receive_window: ReceiveWindow::new(),
            split_handler: SplitPacketHandler::new(),
        }
    }

    /// Handles an incoming datagram for this session.
    pub async fn handle_incoming(&mut self, data: &[u8], now: Instant) {
        self.last_activity = now;
        let mut reader = bytes::Bytes::copy_from_slice(data);

        if reader.is_empty() {
            trace!("[{}] Ignoring empty datagram", self.address);
            return;
        }

        let packet_id = reader.get_u8();

        // Re-check if packet ID looks like ACK/NACK or standard datagram
        // Need to reset reader pointer if it was ACK/NACK for AckNack::decode
        let is_ack_nack =
            packet_id == protocol::ack::ACK_HEADER || packet_id == protocol::ack::NACK_HEADER;
        reader = bytes::Bytes::copy_from_slice(data); // Reset reader for full processing

        if is_ack_nack {
            self.handle_ack_nack(&mut reader).await;
        } else if (packet_id & protocol::datagram::FLAG_VALID) != 0 {
            match Datagram::decode(&mut reader) {
                Ok(datagram) => self.handle_datagram(datagram).await,
                Err(e) => {
                    warn!(
                         "[{}] Failed to decode datagram: {}",
                         self.address, e
                    );
                }
            }
        } else {
            // Handle specific unconnected or other packet types if needed during connected state
            // For now, only log unexpected packets if in connected state
            if self.is_connected() {
                warn!(
                     "[{}] Received unexpected packet ID {:#04x} while connected",
                     self.address, packet_id
                 );
            } else {
                // Handle specific connection phase packets
                match packet_id {
                    protocol::CONNECTION_REQUEST if self.state == SessionState::Connecting => {
                        self.handle_connection_request().await;
                    },
                    protocol::NEW_INCOMING_CONNECTION if self.state == SessionState::Handshaking => {
                        self.handle_post_handshake_connection().await;
                    }
                    protocol::DISCONNECT_NOTIFICATION => {
                        self.handle_disconnect_notification().await;
                    },
                    _ => {
                        warn!("[{}] Received unexpected packet ID {:#04x} in state {:?} ", self.address, packet_id, self.state);
                    }

                }
            }
        }
    }

    /// Handles a received ACK or NACK packet.
    async fn handle_ack_nack(&mut self, reader: &mut bytes::Bytes) {
        let header = reader.get_u8(); // Consume the header byte
        let is_nack = header == protocol::ack::NACK_HEADER;

        match protocol::AckNack::decode(reader, is_nack) {
            Ok(acknack) => {
                if acknack.is_nack {
                    trace!("[{}] Received NACK for {:?} seq nums", self.address, acknack.records);
                    self.send_window.handle_nack(&acknack.records).await;
                } else {
                    trace!("[{}] Received ACK for {:?} seq nums", self.address, acknack.records);
                    self.send_window.handle_ack(&acknack.records, Instant::now()).await;
                }
            }
            Err(e) => warn!("[{}] Failed to decode {}: {}", self.address, if is_nack {"NACK"} else {"ACK"}, e),
        }
    }

    /// Handles a received data datagram.
    async fn handle_datagram(&mut self, datagram: Datagram) {
        trace!("[{}] Handling datagram #{}", self.address, datagram.sequence_number);
        if let Some(packets) = self.receive_window.handle_datagram(datagram) {
            for encap_packet in packets {
                self.handle_encapsulated(encap_packet).await;
            }
        }
    }

    /// Handles a single, potentially split, encapsulated packet.
    async fn handle_encapsulated(&mut self, packet: EncapsulatedPacket) {
        let complete_payload = if packet.is_split {
            match self.split_handler.handle_packet(packet) {
                Ok(Some(payload)) => Some(payload), // Reassembly complete
                Ok(None) => None,                  // Need more parts
                Err(e) => {
                    warn!("[{}] Error handling split packet: {}", self.address, e);
                    // Possibly disconnect or ignore based on error severity
                    None
                }
            }
        } else {
            Some(packet.buffer) // Not split, payload is ready
        };

        if let Some(payload) = complete_payload {
            // --- Here, you'd process the actual game packet payload ---
            // For now, just log it. Replace with actual game logic dispatch.
            debug!("[{}] Received game packet: {:02X?}", self.address, payload);

            // Example: Trigger event, pass to game state, etc.
            // self.game_handler.process(payload).await;
        }
    }

    /// Handles the `ConnectionRequest` packet during the connection phase.
    async fn handle_connection_request(&mut self) {
        trace!("[{}] Handling Connection Request", self.address);
        if self.state == SessionState::Connecting {
            // Send ConnectionRequestAccepted
            let mut writer = BytesMut::new();
            writer.write_u8(protocol::CONNECTION_REQUEST_ACCEPTED).unwrap();
            protocol::write_address(&mut writer, &self.address).expect("Failed to write address");
            writer.write_u16_be(0).unwrap(); // System Index
            // TODO: RakNet V10 adds 20 system addresses here - need to decide how/if to implement this
            for _ in 0..10 { // Placeholder for 10 addresses like PMMP/Cloudburst (adjust number if needed)
                let dummy_addr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
                protocol::write_address(&mut writer, &dummy_addr).expect("Failed to write dummy address");
            }
            writer.write_i64_be(0); // TODO: Request time (use received ping time? Requires storing from ping)
            writer.write_i64_be(Instant::now().elapsed().as_millis() as i64); // Current time


            // Send directly, no reliability needed for this response? (Check RakNet spec)
            // Or should this be queued reliably? Queuing reliably seems safer.
            self.send_internal_packet(Bytes::from(writer.freeze()), Reliability::ReliableOrdered).await;


            // Transition state
            self.state = SessionState::Handshaking;
            debug!("[{}] Sent Connection Request Accepted, state -> Handshaking", self.address);
        } else {
            warn!("[{}] Received Connection Request in unexpected state {:?}", self.address, self.state);
        }
    }


    /// Called by the server thread after OCREP2 is sent, indicates the server is ready.
    /// This session will now wait for NEW_INCOMING_CONNECTION.
    pub async fn handle_new_incoming_connection(&mut self) {
        // We don't strictly *receive* NEW_INCOMING_CONNECTION from the client in this model.
        // OCREP2 acts as the server's signal to create the session.
        // When the client receives OCREP2, it sends CONNECTION_REQUEST.
        // When we receive CONNECTION_REQUEST, we send CONNECTION_REQUEST_ACCEPTED.
        // The *client* sends NEW_INCOMING_CONNECTION to itself conceptually
        // when it receives CONNECTION_REQUEST_ACCEPTED.

        // For the server, receiving CONNECTION_REQUEST and sending
        // CONNECTION_REQUEST_ACCEPTED means we move to the Handshaking state,
        // waiting for the *client* to confirm *its* connection is ready by sending
        // its first reliable packets (like its own NEW_INCOMING_CONNECTION, which we ignore, or game data).

        // A simpler approach is sometimes taken where receiving the *first* reliable datagram (like a game packet)
        // after sending CONNECTION_REQUEST_ACCEPTED moves the server-side session to Connected.
        // For now, we have a dedicated state 'Handshaking' which waits for the client's response implicitly.

        // The `handle_post_handshake_connection` will finalize the state transition
        // when the client sends its NEW_INCOMING_CONNECTION (or first game packet).

        debug!("[{}] Session setup initiated after OCREP2", self.address);
        // State remains Connecting until handle_connection_request is called.
    }

    /// Handles the `NewIncomingConnection` packet sent by the client *after* it receives `ConnectionRequestAccepted`.
    /// This typically marks the connection as fully established on the server side.
    async fn handle_post_handshake_connection(&mut self) {
        trace!("[{}] Handling New Incoming Connection (post-handshake)", self.address);
        if self.state == SessionState::Handshaking {
            self.state = SessionState::Connected;
            debug!("[{}] Connection established, state -> Connected", self.address);
            // TODO: Notify the main application that a new client has connected.
            // Example: self.server_event_sender.send(ServerEvent::Connect(self.address, self.client_guid));
        } else {
            warn!("[{}] Received New Incoming Connection in unexpected state {:?}", self.address, self.state);
        }
    }

    /// Handles a DisconnectNotification packet from the client.
    async fn handle_disconnect_notification(&mut self) {
        debug!("[{}] Received disconnect notification from client.", self.address);
        self.state = SessionState::Disconnected;
        // No need to send ACK for this, client is gone.
        // TODO: Notify main application of disconnection.
        // Example: self.server_event_sender.send(ServerEvent::Disconnect(self.address, DisconnectReason::ClientDisconnect));
    }

    /// Performs periodic updates for the session (e.g., sending ACKs).
    pub async fn tick(&mut self, now: Instant) -> Result<(), RakNetError>{
        // 1. Send queued ACKs/NACKs
        if let Some(ack_packet) = self.receive_window.create_ack_packet() {
            let mut writer = BytesMut::new();
            ack_packet.encode(&mut writer)?;
            self.send_raw(&writer).await?; // ACKs are sent unreliably
            trace!("[{}] Sent ACK packet", self.address);
        }
        if let Some(nack_packet) = self.receive_window.create_nack_packet() {
            let mut writer = BytesMut::new();
            nack_packet.encode(&mut writer)?;
            self.send_raw(&writer).await?; // NACKs are sent unreliably
            trace!("[{}] Sent NACK packet", self.address);
        }


        // 2. Resend lost packets identified by the SendWindow
        self.send_window.tick(now).await;


        // 3. Send queued reliable packets
        while let Some(datagram) = self.send_window.get_next_datagram() {
            trace!("[{}] Sending datagram #{}", self.address, datagram.sequence_number);
            let mut buffer = BytesMut::new();
            datagram.encode(&mut buffer)?;
            self.send_raw(&buffer).await?; // Send the fully encoded datagram
        }

        // 4. Check for timeout (handled by server cleanup task now)
        // if now.duration_since(self.last_activity) > Duration::from_secs(30) { // Configurable timeout
        //     debug!("[{}] Session timed out", self.address);
        //     self.state = SessionState::Disconnected;
        //     // TODO: Notify disconnect
        //     return Err(RakNetError::SessionTimeout);
        // }

        Ok(())
    }

    /// Sends a raw byte slice over the UDP socket to the client.
    async fn send_raw(&self, data: &[u8]) -> std::io::Result<()> {
        self.socket.send_to(data, self.address).await?;
        Ok(())
    }

    /// Sends an internal RakNet packet (like ConnectionRequestAccepted).
    /// This takes care of encapsulating the packet.
    async fn send_internal_packet(&mut self, payload: bytes::Bytes, reliability: Reliability) {
        self.send_window.queue_packet(payload, reliability, Some(0)); // Use channel 0 for internal stuff
        // Tick might send it, or call get_next_datagram explicitly if immediate send needed
        // For now, rely on the next tick.
    }

    /// Sends a game packet (payload) with the specified reliability.
    pub async fn send_packet(&mut self, payload: bytes::Bytes, reliability: Reliability, channel: u8) {
        if !self.is_connected() {
            warn!("[{}] Attempted to send packet while not fully connected (state: {:?})", self.address, self.state);
            return; // Don't send if not connected
        }
        trace!("[{}] Queuing game packet ({} bytes, {:?}, channel {})", self.address, payload.len(), reliability, channel);
        self.send_window.queue_packet(payload, reliability, Some(channel));
    }

    /// Checks if the session has timed out.
    pub fn is_timed_out(&self, now: Instant, timeout_duration: Duration) -> bool {
        self.state != SessionState::Disconnected && now.duration_since(self.last_activity) > timeout_duration
    }

    /// Checks if the session is fully connected.
    pub fn is_connected(&self) -> bool {
        self.state == SessionState::Connected
    }

    /// Checks if the session is disconnected.
    pub fn is_disconnected(&self) -> bool {
        self.state == SessionState::Disconnected
    }

}