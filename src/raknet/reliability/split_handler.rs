// src/raknet/reliability/split_handler.rs
use crate::raknet::protocol::{EncapsulatedPacket, RakNetError};
use bytes::Bytes;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tracing::{debug, trace, warn};


// Maximum number of concurrent split packets being reassembled.
const MAX_CONCURRENT_SPLITS: usize = 64;
// Maximum number of parts a single split packet can have.
const MAX_SPLIT_PARTS: u32 = 128; // Should be less than message_index limit if relevant
// Maximum time to wait for all parts of a split packet before discarding.
const SPLIT_PACKET_TIMEOUT: Duration = Duration::from_secs(30);


/// Tracks the reassembly state of a single split packet.
#[derive(Debug)]
struct SplitPacketReassembly {
    expected_count: u32,
    /// Stores received parts. Option is None if the part hasn't arrived yet.
    parts: Vec<Option<Bytes>>,
    /// Total size of all parts combined (estimated or actual).
    total_size: usize,
    /// Timestamp of the first part received. Used for timeout.
    first_part_time: Instant,
    /// Metadata from the first received part needed for reconstruction
    base_packet_info: EncapsulatedPacket
}

/// Handles reassembly of split packets.
#[derive(Debug)]
pub struct SplitPacketHandler {
    /// Maps split_id to its reassembly tracker.
    reassembly_map: HashMap<u16, SplitPacketReassembly>,
}


impl SplitPacketHandler {
    pub fn new() -> Self {
        Self {
            reassembly_map: HashMap::new(),
        }
    }


    /// Handles an incoming encapsulated packet that is marked as split.
    /// Returns `Ok(Some(Bytes))` if the packet is complete, `Ok(None)` if more parts are needed,
    /// or `Err(RakNetError)` if the packet is invalid or limits are exceeded.
    pub fn handle_packet(&mut self, packet: EncapsulatedPacket) -> Result<Option<Bytes>, RakNetError> {
        if !packet.is_split {
            // This function should only be called for split packets
            return Err(RakNetError::InternalError("handle_packet called on non-split packet".to_string()));
        }


        let split_id = packet.split_id.ok_or(RakNetError::InvalidSplitPacket("Missing split_id".to_string()))?;
        let split_count = packet.split_count.ok_or(RakNetError::InvalidSplitPacket("Missing split_count".to_string()))?;
        let split_index = packet.split_index.ok_or(RakNetError::InvalidSplitPacket("Missing split_index".to_string()))?;


        // --- Input Validation ---
        if split_count == 0 || split_count > MAX_SPLIT_PARTS {
            return Err(RakNetError::InvalidSplitPacket(format!("Invalid split_count: {}", split_count)));
        }
        if split_index >= split_count {
            return Err(RakNetError::InvalidSplitPacket(format!(
                "split_index ({}) >= split_count ({})",
                split_index, split_count
            )));
        }


        // --- Get or Create Reassembly Tracker ---
        let reassembly = match self.reassembly_map.get_mut(&split_id) {
            Some(existing) => {
                // Validate consistency
                if existing.expected_count != split_count {
                    // Inconsistent split count for the same ID, discard old one? Or error out? Error seems safer.
                    warn!("Inconsistent split_count for split_id {}: got {}, expected {}", split_id, split_count, existing.expected_count);
                    self.reassembly_map.remove(&split_id); // Clean up bad state
                    return Err(RakNetError::InvalidSplitPacket(format!(
                        "Inconsistent split_count ({} vs {})",
                        split_count, existing.expected_count
                    )));
                }
                // Check timeout
                if existing.first_part_time.elapsed() > SPLIT_PACKET_TIMEOUT {
                    debug!("Split packet {} timed out, discarding.", split_id);
                    self.reassembly_map.remove(&split_id);
                    // Need to potentially create a *new* tracker if this packet starts a new split with the same ID later
                    // For now, return None as this packet belongs to the timed-out sequence
                    return Ok(None);
                }
                existing
            }
            None => {
                // Check concurrent limit *before* inserting
                if self.reassembly_map.len() >= MAX_CONCURRENT_SPLITS {
                    // Find and remove the oldest entry? Or just reject new ones? Rejecting is simpler.
                    warn!("Max concurrent splits ({}) reached, dropping new split packet id {}", MAX_CONCURRENT_SPLITS, split_id);
                    return Err(RakNetError::TooManySplitPackets);
                }
                trace!("Starting reassembly for split packet id {}", split_id);
                // Clone necessary info from the first packet
                let base_info = EncapsulatedPacket {
                    reliability: packet.reliability,
                    is_split: false, // The final packet isn't split
                    message_index: packet.message_index,
                    sequence_index: packet.sequence_index,
                    order_index: packet.order_index,
                    order_channel: packet.order_channel,
                    split_count: None, split_id: None, split_index: None, // Reset split info
                    buffer: Bytes::new(), // Payload will be built
                    ack_id: packet.ack_id,
                };

                self.reassembly_map.insert(
                    split_id,
                    SplitPacketReassembly {
                        expected_count: split_count,
                        parts: vec![None; split_count as usize],
                        total_size: 0, // Calculate as parts arrive
                        first_part_time: Instant::now(),
                        base_packet_info: base_info
                    },
                );
                self.reassembly_map.get_mut(&split_id).unwrap() // Should exist now
            }
        };


        // --- Store the part ---
        let part_index_usize = split_index as usize;
        if reassembly.parts[part_index_usize].is_none() {
            reassembly.parts[part_index_usize] = Some(packet.buffer);
            reassembly.total_size += reassembly.parts[part_index_usize].as_ref().unwrap().len();
        } else {
            // Duplicate part, ignore it.
            trace!(
                 "Received duplicate part {} for split packet id {}",
                 split_index,
                split_id
             );
            return Ok(None);
        }


        // --- Check for Completion ---
        if reassembly.parts.iter().all(Option::is_some) {
            trace!("Split packet id {} complete.", split_id);
            let reassembly_info = self.reassembly_map.remove(&split_id).unwrap(); // Remove from map


            // TODO: Maybe check total_size against MTU*split_count as sanity check?

            // Reconstruct the final payload
            let mut final_buffer = BytesMut::with_capacity(reassembly_info.total_size);
            for part_option in reassembly_info.parts {
                final_buffer.extend_from_slice(&part_option.unwrap()); // We know they are all Some now
            }


            // Create the final encapsulated packet using info from the first part
            let mut final_packet = reassembly_info.base_packet_info; // Take ownership
            final_packet.buffer = final_buffer.freeze();


            // For now, we only return the buffer. The caller (session) will handle it.
            Ok(Some(final_packet.buffer))
            // Alternatively, return the reconstructed EncapsulatedPacket:
            // Ok(Some(final_packet)) -> then session would need to handle this packet type
        } else {
            // Still waiting for more parts
            Ok(None)
        }
    }


    /// Cleans up timed-out partial split packets. Call this periodically.
    pub fn cleanup_timeouts(&mut self) {
        let now = Instant::now();
        let initial_len = self.reassembly_map.len();
        self.reassembly_map.retain(|split_id, reassembly| {
            let retain = now.duration_since(reassembly.first_part_time) <= SPLIT_PACKET_TIMEOUT;
            if !retain {
                debug!("Timing out incomplete split packet id {}", split_id);
            }
            retain
        });
        if initial_len > self.reassembly_map.len() {
            trace!("Cleaned up {} timed-out split packets.", initial_len - self.reassembly_map.len());
        }
    }

}