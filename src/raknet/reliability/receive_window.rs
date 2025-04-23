// src/raknet/reliability/receive_window.rs
use crate::raknet::protocol::ack::{self, AckNack, AckNackRecord};
use crate::raknet::protocol::Datagram;
use crate::raknet::protocol::EncapsulatedPacket;
use std::collections::{BTreeSet, HashMap, VecDeque};
use tracing::warn;

const MAX_RECV_WINDOW_SIZE: u32 = 2048; // Max packets to track for ACKs/NACKs

/// Handles reliable receiving, duplicate detection, and ACK/NACK generation.
#[derive(Debug)]
pub struct ReceiveWindow {
    /// The next sequence number expected. Datagrams with lower numbers are duplicates.
    expected_sequence_number: u32,
    /// Set of sequence numbers received within the current window. Used for duplicate checks and NACK generation.
    received_datagrams: BTreeSet<u32>, // BTreeSet keeps them sorted, useful for NACKs
    /// Highest sequence number received so far within the current window start/end bounds (not absolute highest).
    highest_received_in_window: u32, // Helps calculate missing ranges
    /// Packets that arrived out of order, waiting for missing packets. Keyed by order_channel.
    // Use VecDeque for efficient push/pop and indexing if needed? Check if needed. HashMap seems simpler for lookups.
    ordering_queues: HashMap<u8, HashMap<u32, EncapsulatedPacket>>,
    /// Next expected order index for each channel.
    next_order_indices: HashMap<u8, u32>,
    /// Last acknowledged sequence numbers to generate ACK packet
    ack_queue: Vec<u32>,
    /// Missing sequence numbers to generate NACK packet
    nack_queue: BTreeSet<u32>, // BTreeSet to easily track ranges and avoid duplicates
}

impl ReceiveWindow {
    pub fn new() -> Self {
        Self {
            expected_sequence_number: 0,
            received_datagrams: BTreeSet::new(),
            highest_received_in_window: 0,
            ordering_queues: HashMap::new(),
            next_order_indices: HashMap::new(),
            ack_queue: Vec::new(),
            nack_queue: BTreeSet::new(),
        }
    }

    /// Handles an incoming datagram.
    /// Returns a Vec of ready-to-process encapsulated packets if the datagram completes any sequences/orders.
    pub fn handle_datagram(&mut self, datagram: Datagram) -> Option<Vec<EncapsulatedPacket>> {
        let seq = datagram.sequence_number;

        // 1. Duplicate or Out-of-Window Check
        if seq < self.expected_sequence_number || self.received_datagrams.contains(&seq) {
            warn!("Duplicate or old datagram received: #{}", seq);
            // Optionally send ACK again for duplicates? RakNet might do this. For now, just ignore.
            return None; // Duplicate or too old
        }

        // Window size check (rough estimate to prevent DoS)
        if let Some(max_in_window) = self.received_datagrams.iter().next_back() {
            if seq.wrapping_sub(*max_in_window) > MAX_RECV_WINDOW_SIZE {
                warn!("Datagram #{} outside window size limit, ignoring.", seq);
                return None;
            }
        } else if seq.wrapping_sub(self.expected_sequence_number) > MAX_RECV_WINDOW_SIZE {
            warn!("Datagram #{} outside window size limit, ignoring.", seq);
            return None;
        }


        // 2. Add to received set and ACK queue
        self.received_datagrams.insert(seq);
        self.ack_queue.push(seq);
        // Remove from NACK queue if it was there (client retransmitted)
        self.nack_queue.remove(&seq);


        self.highest_received_in_window = std::cmp::max(self.highest_received_in_window, seq);

        // 3. Check for missing packets and queue NACKs
        if seq > self.expected_sequence_number {
            // There's a gap, mark packets between expected and current as potentially missing
            for missing_seq in self.expected_sequence_number..seq {
                if !self.received_datagrams.contains(&missing_seq) {
                    // Add to NACK unless already NACKed recently? Add simple logic first.
                    self.nack_queue.insert(missing_seq);
                }
            }
        }

        // 4. Try to advance the expected sequence number
        while self.received_datagrams.contains(&self.expected_sequence_number) {
            self.received_datagrams.remove(&self.expected_sequence_number);
            // We might have NACKed this earlier, clean up NACK queue just in case
            // (though ideally it should be removed when inserted into received_datagrams)
            self.nack_queue.remove(&self.expected_sequence_number);
            self.expected_sequence_number = self.expected_sequence_number.wrapping_add(1);
        }


        // 5. Process encapsulated packets
        let mut ready_packets = Vec::new();
        for packet in datagram.packets {
            if packet.reliability.needs_ordering_info() {
                if let Some(channel) = packet.order_channel {
                    if let Some(order_index) = packet.order_index {
                        let queue = self.ordering_queues.entry(channel).or_default();
                        let next_expected_index = self.next_order_indices.entry(channel).or_insert(0);

                        if order_index == *next_expected_index {
                            // Got the expected ordered packet
                            ready_packets.push(packet);
                            *next_expected_index = next_expected_index.wrapping_add(1);

                            // Process any queued packets for this channel that are now in order
                            while let Some(queued_packet) = queue.remove(next_expected_index) {
                                ready_packets.push(queued_packet);
                                *next_expected_index = next_expected_index.wrapping_add(1);
                            }
                        } else if order_index > *next_expected_index {
                            // Out of order packet, queue it
                            // TODO: Add limits to queue size per channel
                            queue.insert(order_index, packet);
                        } else {
                            // Duplicate ordered packet, ignore
                            warn!(
                                "[ch{}] Ignoring duplicate ordered packet #{}",
                                 channel, order_index
                             );
                        }
                    } else {
                        warn!(
                            "Packet has ordering channel but missing order index: {:?}",
                            packet
                        );
                    }
                } else {
                    warn!(
                        "Packet needs ordering info but missing channel: {:?}",
                         packet
                     );
                }
            } else {
                // Unordered packet, process immediately
                ready_packets.push(packet);
            }
        }


        if ready_packets.is_empty() {
            None
        } else {
            Some(ready_packets)
        }
    }

    /// Creates an ACK packet if there are acknowledged sequence numbers pending.
    pub fn create_ack_packet(&mut self) -> Option<AckNack> {
        if self.ack_queue.is_empty() {
            return None;
        }


        // Sort sequence numbers to optimize into ranges
        self.ack_queue.sort_unstable();
        // Optimization might remove duplicates implicitly, or do it explicitly:
        // self.ack_queue.dedup();


        let records = ack::optimize_ack_nack_records(&self.ack_queue);
        self.ack_queue.clear(); // Clear after processing

        Some(AckNack {
            is_nack: false,
            records,
        })
    }

    /// Creates a NACK packet if there are missing sequence numbers pending.
    pub fn create_nack_packet(&mut self) -> Option<AckNack> {
        if self.nack_queue.is_empty() {
            return None;
        }

        // Convert BTreeSet to sorted Vec for optimization function
        let sorted_nacks: Vec<u32> = self.nack_queue.iter().cloned().collect();
        let records = ack::optimize_ack_nack_records(&sorted_nacks);


        // Clear NACK queue *after* creating packet to avoid re-NACKing immediately
        // Or manage NACK resend timing more explicitly later. For now, clear.
        self.nack_queue.clear();


        Some(AckNack {
            is_nack: true,
            records,
        })
    }
}