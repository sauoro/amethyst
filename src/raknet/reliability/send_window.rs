// src/raknet/reliability/send_window.rs
use crate::raknet::protocol::ack::{AckNackRecord, ACK, NACK};
use crate::raknet::protocol::{Datagram, EncapsulatedPacket, Reliability};
use std::collections::{BTreeMap, BinaryHeap, HashMap, VecDeque};
use std::time::{Duration, Instant};
use tracing::{trace, warn};

// Heuristic constants for congestion control and retransmission
const RTT_ALPHA: f64 = 0.125; // Smoothing factor for RTT calculation (EWMA alpha)
const RTT_BETA: f64 = 0.25; // Smoothing factor for RTT deviation (EWMA beta)
const MIN_RTO: Duration = Duration::from_millis(100); // Minimum retransmission timeout
const MAX_RTO: Duration = Duration::from_secs(5); // Maximum retransmission timeout
const INITIAL_RTO: Duration = Duration::from_millis(500); // Initial RTO before first RTT measurement
const RESEND_DELAY_FACTOR: f64 = 1.5; // Multiplier for RTT to determine base resend delay

// Congestion Control Constants
const INITIAL_CWND_PACKETS: u32 = 2; // Initial congestion window size in packets (often 2-4 * MTU)
const MIN_CWND_BYTES: usize = 1400; // Minimum congestion window in bytes (roughly 1 MTU)


/// Manages reliable sending, congestion control, packet loss detection, and retransmissions.
#[derive(Debug)]
pub struct SendWindow {
    /// Next sequence number to assign to an outgoing datagram.
    next_sequence_number: u32,
    /// Next message index to assign to a reliable encapsulated packet.
    next_message_index: u32,
    /// Next order index for each channel.
    next_order_indices: HashMap<u8, u32>,
    /// Packets waiting to be sent, organized by priority (though basic VecDeque for now).
    // TODO: Implement priority queue if needed (e.g., BinaryHeap with priority wrapper)
    packet_queue: VecDeque<EncapsulatedPacket>,
    /// Datagrams sent but not yet ACKed. Key: sequence_number. Value: (datagram, send_time, has_critical_packet)
    pending_datagrams: BTreeMap<u32, (Datagram, Instant, bool)>,
    /// Lost sequence numbers that need resending immediately (due to NACK).
    // Using BTreeSet for ordered iteration might be beneficial if resending in order matters.
    resend_queue: BTreeSet<u32>,

    /// Round Trip Time (smoothed estimate).
    srtt: Option<Duration>,
    /// RTT Variance (smoothed estimate).
    rtt_var: Option<Duration>,
    /// Retransmission Timeout duration.
    rto: Duration,
    /// Maximum Transmission Unit for creating datagrams.
    mtu: usize,

    /// Congestion Window size in bytes.
    cwnd: usize,
    /// Slow Start Threshold in bytes.
    ssthresh: usize,
    /// Bytes currently in flight (sent but not ACKed).
    bytes_in_flight: usize,

    /// Tracks ACK IDs for packets needing ACK confirmation
    ack_trackers: HashMap<u32, AckTracker>,

    /// Instant when the last ACK was received, used for rate control? (Maybe not needed yet)
    last_ack_time: Instant,

    /// Stores a datagram temporarily if it can't be sent due to congestion.
    congestion_hold: Option<Datagram>,

}

#[derive(Debug)]
struct AckTracker {
    sequence_numbers: BTreeSet<u32>,
    // Could add send time or other metadata if needed for more complex logic
}


impl SendWindow {
    pub fn new(mtu: usize) -> Self {
        Self {
            next_sequence_number: 0,
            next_message_index: 0,
            next_order_indices: HashMap::new(),
            packet_queue: VecDeque::new(),
            pending_datagrams: BTreeMap::new(),
            resend_queue: BTreeSet::new(),
            srtt: None,
            rtt_var: None,
            rto: INITIAL_RTO,
            mtu,
            // Adjust initial CWND based on MTU, e.g., 2 * MTU
            cwnd: std::cmp::max(INITIAL_CWND_PACKETS as usize * mtu, MIN_CWND_BYTES),
            // Initialize ssthresh high, effectively starting in slow start
            ssthresh: usize::MAX,
            bytes_in_flight: 0,
            ack_trackers: HashMap::new(),
            last_ack_time: Instant::now(), // Initialize to now
            congestion_hold: None,

        }
    }

    /// Queues a payload to be sent reliably.
    pub fn queue_packet(&mut self, buffer: bytes::Bytes, reliability: Reliability, channel: Option<u8>) -> Option<u32>{
        let mut packet = EncapsulatedPacket {
            reliability,
            is_split: false,
            message_index: None,
            sequence_index: None,
            order_index: None,
            order_channel: None,
            split_count: None,
            split_id: None,
            split_index: None,
            buffer,
            ack_id: None, // Ack ID assigned later if needed
        };

        // Assign indices based on reliability
        if reliability.is_reliable() {
            packet.message_index = Some(self.next_message_index);
            self.next_message_index = self.next_message_index.wrapping_add(1);
        }

        if reliability.needs_ordering_info() {
            let ch = channel.unwrap_or(0); // Default to channel 0 if none provided
            packet.order_channel = Some(ch);
            let order_index_ref = self.next_order_indices.entry(ch).or_insert(0);
            packet.order_index = Some(*order_index_ref);
            if reliability.is_ordered() {
                // Only increment order index for ReliableOrdered/UnreliableOrdered
                *order_index_ref = order_index_ref.wrapping_add(1);
            }
            // sequence_index is usually assigned per datagram/tick in some RakNet versions,
            // let's omit it for encapsulated packets unless required? Seems unnecessary for this layer.
        }


        // TODO: Implement packet splitting if packet.buffer.len() > allowed payload size (mtu - headers)
        // For now, assume packets are pre-split or fit within MTU.
        // If splitting: Generate split_id, split_count, split_index for each fragment.


        self.packet_queue.push_back(packet);

        // Return message_index if packet is reliable, can be used as an ACK ID
        packet.message_index
    }


    /// Assigns an Ack ID to the last queued reliable packet with the given message index.
    pub fn assign_ack_id(&mut self, message_index: u32, ack_id: u32) {
        // Find the packet(s) with this message index in the queue (could be split)
        // And update its ack_id field. This requires iterating, maybe store message_idx -> queue_pos map?
        // Or more simply, rely on the caller using the returned message_index immediately after queueing.
        if let Some(packet) = self.packet_queue.iter_mut().rev().find(|p| p.message_index == Some(message_index)) {
            packet.ack_id = Some(ack_id);

            // Start tracking this ack_id
            let tracker = self.ack_trackers.entry(ack_id).or_insert_with(|| AckTracker {
                sequence_numbers: BTreeSet::new(),
            });
            // We don't know the sequence number *yet*. It will be added when the datagram is sent.
        } else {
            warn!("Could not find packet with message_index {} to assign ack_id {}", message_index, ack_id);
        }
    }


    /// Returns the next datagram to be sent, if any, respecting MTU and congestion window.
    pub fn get_next_datagram(&mut self) -> Option<Datagram> {

        // Prioritize retransmissions held due to previous congestion
        if self.bytes_in_flight < self.cwnd {
            if let Some(mut held_datagram) = self.congestion_hold.take() {
                let datagram_size = held_datagram.size_for_congestion_control.unwrap_or(0);
                // Assume retransmissions don't count towards congestion initially?
                // Or add check `self.bytes_in_flight + datagram_size <= self.cwnd`? Revisit congestion rules.
                trace!("Sending held datagram #{}", held_datagram.sequence_number);
                held_datagram.send_time = Some(Instant::now()); // Update send time
                let has_critical = held_datagram.packets.iter().any(|p| p.reliability.is_reliable());
                self.pending_datagrams.insert(held_datagram.sequence_number, (held_datagram.clone(), Instant::now(), has_critical));
                // self.bytes_in_flight += datagram_size; // Account for retransmission in flight? Yes.
                self.bytes_in_flight = self.bytes_in_flight.saturating_add(datagram_size);
                return Some(held_datagram);
            }
        }


        // Process immediate resends (from NACKs)
        if let Some(seq_to_resend) = self.resend_queue.pop_first() {
            if let Some((datagram, _send_time, _critical)) = self.pending_datagrams.get(&seq_to_resend) {
                // Check congestion window before resending
                let datagram_size = datagram.size_for_congestion_control.unwrap_or(0);
                if self.bytes_in_flight + datagram_size <= self.cwnd {
                    let mut resend_datagram = datagram.clone();
                    resend_datagram.send_time = Some(Instant::now()); // Update send time
                    // No need to re-insert into pending_datagrams, just update the existing one's time?
                    // Or should we track retransmits separately?
                    // For now, just send it. If it gets ACKed, the original entry will be removed.
                    // Update the original entry's time maybe?
                    if let Some((_, time_ref, _)) = self.pending_datagrams.get_mut(&seq_to_resend) {
                        *time_ref = Instant::now();
                    }
                    // Add to flight bytes *again*? No, it's already counted.

                    // We actually need to re-assign sequence number for retransmissions? No, raknet resends with original.
                    trace!("Resending datagram #{} due to NACK", seq_to_resend);
                    return Some(resend_datagram);
                } else {
                    // Can't resend now due to congestion, put it back and hold
                    trace!("Congestion prevented resending datagram #{}", seq_to_resend);
                    self.resend_queue.insert(seq_to_resend); // Put it back
                    return None; // Wait for window to open
                }
            } else {
                warn!("Datagram #{} requested for resend (NACK) not found in pending", seq_to_resend);
            }
        }

        // If nothing to resend and queue is empty, nothing to do
        if self.packet_queue.is_empty() {
            return None;
        }


        // Build a new datagram from the packet queue
        let mut datagram = Datagram::new(self.next_sequence_number);
        let mut current_size = 1 + 3; // Header size
        let mut has_critical_packet = false;

        while let Some(packet) = self.packet_queue.front() {
            let packet_size = packet.calculate_size();
            if current_size + packet_size > self.mtu {
                break; // Exceeds MTU
            }

            // Check congestion window based on potential new size
            let final_datagram_size = current_size + packet_size;
            if self.bytes_in_flight + final_datagram_size > self.cwnd {
                trace!(
                     "Congestion window limit ({}/{}) preventing sending more packets in datagram #{}",
                     self.bytes_in_flight + final_datagram_size,
                     self.cwnd,
                    datagram.sequence_number
                );
                break; // Congestion window full
            }


            // Safe to add packet
            let packet = self.packet_queue.pop_front().unwrap(); // We know it exists from front()

            if packet.reliability.is_reliable() {
                has_critical_packet = true;
            }

            // If this packet requires an ACK, track its datagram sequence number
            if let Some(ack_id) = packet.ack_id {
                if let Some(tracker) = self.ack_trackers.get_mut(&ack_id) {
                    tracker.sequence_numbers.insert(datagram.sequence_number);
                }
            }

            current_size += packet_size;
            datagram.packets.push(packet);
        }


        if datagram.packets.is_empty() {
            // Could not fit even the first packet, or queue was emptied by previous checks.
            return None;
        }

        // Store datagram info for potential retransmission and RTT calculation
        let now = Instant::now();
        datagram.send_time = Some(now);
        datagram.size_for_congestion_control = Some(current_size); // Use the final calculated size
        self.pending_datagrams.insert(datagram.sequence_number, (datagram.clone(), now, has_critical_packet));
        self.bytes_in_flight = self.bytes_in_flight.saturating_add(current_size);

        self.next_sequence_number = self.next_sequence_number.wrapping_add(1);

        Some(datagram)
    }


    /// Processes an ACK received from the peer.
    pub async fn handle_ack(&mut self, records: &[AckNackRecord], now: Instant) {
        let mut acked_seq_nums = BTreeSet::new(); // Keep track to update ACK trackers
        let mut total_acked_bytes = 0;
        let mut latest_rtt = None;


        for record in records {
            match record {
                AckNackRecord::Single(seq) => {
                    if let Some((acked_datagram, send_time, _)) = self.pending_datagrams.remove(seq) {
                        total_acked_bytes += acked_datagram.size_for_congestion_control.unwrap_or(0);
                        let rtt = now.duration_since(send_time);
                        self.update_rto(rtt);
                        latest_rtt = Some(rtt);
                        acked_seq_nums.insert(*seq);
                    }
                }
                AckNackRecord::Range(start, end) => {
                    // Use BTreeMap::range for efficient iteration
                    // Need to collect keys first because remove alters the map during iteration
                    let keys_in_range: Vec<u32> = self.pending_datagrams
                        .range(*start..=*end)
                        .map(|(&k, _)| k)
                        .collect();


                    for seq in keys_in_range {
                        if let Some((acked_datagram, send_time, _)) = self.pending_datagrams.remove(&seq) {
                            total_acked_bytes += acked_datagram.size_for_congestion_control.unwrap_or(0);
                            let rtt = now.duration_since(send_time);
                            self.update_rto(rtt);
                            latest_rtt = Some(rtt);
                            acked_seq_nums.insert(seq);
                        }
                    }
                }
            }
        }


        if total_acked_bytes > 0 {
            self.bytes_in_flight = self.bytes_in_flight.saturating_sub(total_acked_bytes);
            // Adjust congestion window (TCP Reno-like logic for simplicity)
            if self.cwnd < self.ssthresh {
                // Slow Start: Increase CWND exponentially (by bytes_acked, approximating MSS)
                self.cwnd = self.cwnd.saturating_add(total_acked_bytes);
            } else {
                // Congestion Avoidance: Increase linearly
                // self.cwnd += (self.mtu * self.mtu) / self.cwnd; // Classic Reno AIMD (bytes^2 / bytes)
                // Simpler approximation: increase by roughly 1 MSS per RTT.
                // If latest_rtt is available, scale increase. For now, simple add.
                self.cwnd = self.cwnd.saturating_add(self.mtu); // Increase by roughly 1 MTU per ACK event
            }
            trace!(
                "ACK: acked_bytes={}, flight={}, cwnd={}, ssthresh={}",
                total_acked_bytes, self.bytes_in_flight, self.cwnd, self.ssthresh
            );

            // Process ACK trackers
            let mut completed_ack_ids = Vec::new();
            for (&ack_id, tracker) in self.ack_trackers.iter_mut() {
                let mut completed_for_id = true;
                for seq in tracker.sequence_numbers.iter() {
                    if !acked_seq_nums.contains(seq) {
                        completed_for_id = false;
                        break;
                    }
                }
                if completed_for_id {
                    // TODO: Notify listener that ack_id is complete
                    debug!("ACK ID {} confirmed.", ack_id);
                    completed_ack_ids.push(ack_id);
                } else {
                    // Remove the sequence numbers that were acked
                    tracker.sequence_numbers.retain(|seq| !acked_seq_nums.contains(seq));
                }
            }
            for ack_id in completed_ack_ids {
                self.ack_trackers.remove(&ack_id);
            }


        }


        self.last_ack_time = now; // Update last ACK time
    }


    /// Processes a NACK received from the peer.
    pub async fn handle_nack(&mut self, records: &[AckNackRecord]) {
        let mut retransmit_count = 0;
        // Handle NACK: mark specified sequence numbers for immediate resend
        for record in records {
            match record {
                AckNackRecord::Single(seq) => {
                    if self.pending_datagrams.contains_key(seq) && !self.resend_queue.contains(seq){
                        self.resend_queue.insert(*seq);
                        retransmit_count += 1;
                    }
                }
                AckNackRecord::Range(start, end) => {
                    for seq in *start..=*end {
                        if self.pending_datagrams.contains_key(&seq) && !self.resend_queue.contains(&seq) {
                            self.resend_queue.insert(seq);
                            retransmit_count += 1;
                        }
                    }
                }
            }
        }

        // NACK typically triggers congestion control adjustment (TCP Fast Retransmit/Recovery)
        if retransmit_count > 0 {
            self.ssthresh = std::cmp::max(self.cwnd / 2, MIN_CWND_BYTES); // Halve ssthresh (or min)
            self.cwnd = self.ssthresh; // Reset cwnd to new ssthresh (Fast Recovery phase in TCP)
            // More advanced: inflate cwnd during recovery? For now, reset to ssthresh.
            trace!(
                 "NACK: count={}, flight={}, cwnd={}, ssthresh={}",
                retransmit_count, self.bytes_in_flight, self.cwnd, self.ssthresh
            );
        }
    }

    /// Performs periodic checks, like retransmitting timed-out packets.
    pub async fn tick(&mut self, now: Instant) {
        let mut retransmit_timeout = Vec::new();
        for (&seq, (_datagram, send_time, has_critical)) in &self.pending_datagrams {
            if now.duration_since(*send_time) > self.rto && *has_critical {
                // Timeout detected for a datagram containing reliable packets
                if !self.resend_queue.contains(&seq) {
                    trace!(
                        "Retransmission timeout for datagram #{}, RTO={:?}",
                         seq,
                         self.rto
                     );
                    retransmit_timeout.push(seq);
                }
            }
        }


        if !retransmit_timeout.is_empty() {
            // Double RTO (Exponential Backoff) on timeout - simplified version
            self.rto = std::cmp::min(self.rto * 2, MAX_RTO);

            // TCP often halves ssthresh on timeout
            self.ssthresh = std::cmp::max(self.cwnd / 2, MIN_CWND_BYTES);
            self.cwnd = MIN_CWND_BYTES; // Reset cwnd to minimum (more aggressive than just ssthresh)

            trace!(
                 "Timeout: rtx={}, rto={:?}, flight={}, cwnd={}, ssthresh={}",
                retransmit_timeout.len(), self.rto, self.bytes_in_flight, self.cwnd, self.ssthresh
            );


            for seq in retransmit_timeout {
                if !self.resend_queue.contains(&seq) {
                    self.resend_queue.insert(seq);
                }
            }
        }
    }

    /// Updates RTO based on Jacobson/Karels algorithm.
    fn update_rto(&mut self, rtt: Duration) {
        if self.srtt.is_none() {
            // First measurement
            self.srtt = Some(rtt);
            // RTT variance is half the RTT on the first measurement
            self.rtt_var = Some(rtt / 2);
        } else {
            let srtt = self.srtt.unwrap();
            let rtt_var = self.rtt_var.unwrap();


            // RTTVAR = (1 - beta) * RTTVAR + beta * |SRTT - R'|
            // Using f64 for calculation precision
            let rtt_f = rtt.as_secs_f64();
            let srtt_f = srtt.as_secs_f64();
            let var_diff = (srtt_f - rtt_f).abs();
            let new_rtt_var_f = (1.0 - RTT_BETA) * rtt_var.as_secs_f64() + RTT_BETA * var_diff;


            // SRTT = (1 - alpha) * SRTT + alpha * R'
            let new_srtt_f = (1.0 - RTT_ALPHA) * srtt_f + RTT_ALPHA * rtt_f;

            self.srtt = Some(Duration::from_secs_f64(new_srtt_f));
            self.rtt_var = Some(Duration::from_secs_f64(new_rtt_var_f));
        }

        // RTO = SRTT + max(G, K*RTTVAR) where K=4
        let rto_candidate = self.srtt.unwrap() + self.rtt_var.unwrap() * 4;


        // Clamp RTO between MIN_RTO and MAX_RTO
        self.rto = std::cmp::max(MIN_RTO, std::cmp::min(rto_candidate, MAX_RTO));

        // trace!("RTT={}, SRTT={:?}, RTTVAR={:?}, RTO={:?}", rtt.as_millis(), self.srtt, self.rtt_var, self.rto);
    }

}

// BTreeSet is used for resend_queue to automatically handle order if needed
// and prevent duplicate entries efficiently.
