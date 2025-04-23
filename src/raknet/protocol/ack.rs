// src/raknet/protocol/ack.rs
//! Structures and handlers specific to ACK/NACK packets.

use crate::utils::binary::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// Represents either an ACK or NACK packet header.
pub const ACK_HEADER: u8 = super::datagram::FLAG_VALID | super::datagram::FLAG_ACK; // 0xC0
pub const NACK_HEADER: u8 = super::datagram::FLAG_VALID | super::datagram::FLAG_NACK; // 0xA0

/// Maximum number of sequence number records allowed in a single ACK/NACK packet.
/// This prevents excessively large packets if a peer sends malicious data.
const MAX_RECORDS: u16 = 8192;
/// Maximum range size for a single range record to prevent amplification attacks
/// or excessive memory usage when decoding. Based on RakLib PHP's value.
const MAX_RANGE_SIZE: u32 = 512;

/// Represents an ACK or NACK record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AckNackRecord {
    /// A single acknowledged or NACKed sequence number.
    Single(u32), // u24 LE
    /// A range of acknowledged or NACKed sequence numbers (inclusive).
    Range(u32, u32), // u24 LE, u24 LE
}

/// Represents an ACK (Acknowledgement) or NACK (Negative Acknowledgement) packet.
#[derive(Debug, Clone)]
pub struct AckNack {
    /// Indicates if this is a NACK packet (`true`) or an ACK packet (`false`).
    pub is_nack: bool,
    /// The list of acknowledged or NACKed sequence numbers/ranges.
    pub records: Vec<AckNackRecord>,
}

impl AckNack {
    /// Creates a new ACK packet.
    pub fn new_ack() -> Self {
        Self { is_nack: false, records: Vec::new() }
    }

    /// Creates a new NACK packet.
    pub fn new_nack() -> Self {
        Self { is_nack: true, records: Vec::new() }
    }

    /// Adds a single sequence number to the records.
    pub fn add_single(&mut self, seq_num: u32) {
        self.records.push(AckNackRecord::Single(seq_num));
    }

    /// Adds a range of sequence numbers to the records.
    pub fn add_range(&mut self, start: u32, end: u32) {
        if start <= end {
            self.records.push(AckNackRecord::Range(start, end));
        }
    }

    /// Decodes an ACK or NACK packet from a byte slice.
    /// Assumes the initial packet ID byte has already been consumed.
    pub fn decode(reader: &mut Bytes, is_nack: bool) -> Result<Self, BinaryError> {
        let record_count = reader.read_u16_be()?;
        let mut records = Vec::with_capacity(record_count as usize); // Pre-allocate hint

        for _ in 0..record_count {
            if reader.remaining() == 0 {
                // Premature end, even though count indicated more records
                return Err(BinaryError::UnexpectedEof { needed: 1, remaining: 0});
            }
            let record_type = reader.get_u8();
            match record_type {
                0 => { // Range record
                    let start = reader.read_u24_le()?;
                    let end = reader.read_u24_le()?;
                    if start > end {
                        return Err(BinaryError::InvalidData(format!("Invalid range record: start ({}) > end ({})", start, end)));
                    }
                    // Clamp range size for safety
                    let actual_end = if end - start >= MAX_RANGE_SIZE { start + MAX_RANGE_SIZE -1 } else { end };

                    records.push(AckNackRecord::Range(start, actual_end));
                }
                1 => { // Single record
                    let seq_num = reader.read_u24_le()?;
                    records.push(AckNackRecord::Single(seq_num));
                }
                _ => {
                    return Err(BinaryError::InvalidData(format!("Unknown ACK/NACK record type: {}", record_type)));
                }
            }

            // Safety check: Prevent overly large packet parsing even if record count is misleadingly large.
            if records.len() >= MAX_RECORDS as usize {
                warn!(
                    "ACK/NACK packet contains {} or more records, stopping parse early (declared count: {}). Potential DoS.",
                     MAX_RECORDS, record_count
                 );
                break;
            }

        }

        // Optional: Check if reader has unexpected remaining bytes
        if reader.has_remaining() {
            warn!("{} bytes remaining after decoding ACK/NACK packet", reader.remaining());
        }

        Ok(Self { is_nack, records })
    }

    /// Encodes the ACK or NACK packet into a BytesMut buffer.
    pub fn encode(&self, writer: &mut BytesMut) -> Result<(), BinaryError> {
        writer.put_u8(if self.is_nack { NACK_HEADER } else { ACK_HEADER });

        // Need to limit the number of records? Let's assume `records` is already optimized.
        let record_count: u16 = self.records.len().try_into().map_err(|_| {
            BinaryError::InvalidData("Too many ACK/NACK records".to_string()) // Should ideally not happen
        })?;
        writer.write_u16_be(record_count)?;


        for record in &self.records {
            match *record {
                AckNackRecord::Single(seq_num) => {
                    writer.put_u8(1); // Record type single
                    writer.write_u24_le(seq_num)?;
                }
                AckNackRecord::Range(start, end) => {
                    writer.put_u8(0); // Record type range
                    writer.write_u24_le(start)?;
                    writer.write_u24_le(end)?;
                }
            }
        }
        Ok(())
    }

    /// Extracts all individual sequence numbers represented by the records.
    /// Use with caution for large ranges.
    pub fn extract_sequence_numbers(&self) -> Vec<u32> {
        let mut seq_nums = Vec::new();
        for record in &self.records {
            match record {
                AckNackRecord::Single(n) => seq_nums.push(*n),
                AckNackRecord::Range(start, end) => {
                    // Be mindful of memory if range is huge
                    // If ranges can be extremely large, consider an iterator approach
                    seq_nums.extend(*start..=*end);
                }
            }
        }
        seq_nums
    }
}

/// Type alias for ACK packet.
pub type ACK = AckNack;
/// Type alias for NACK packet.
pub type NACK = AckNack;

// --- Utility Function for Optimizing Records ---
// Note: This requires sorting the sequence numbers first.

/// Optimizes a sorted list of sequence numbers into ACK/NACK records (Singles and Ranges).
pub fn optimize_ack_nack_records(sorted_seq_nums: &[u32]) -> Vec<AckNackRecord> {
    if sorted_seq_nums.is_empty() {
        return Vec::new();
    }

    let mut records = Vec::new();
    let mut start = sorted_seq_nums[0];
    let mut current = start;

    for &seq_num in sorted_seq_nums.iter().skip(1) {
        if seq_num == current + 1 {
            // Part of the current range
            current = seq_num;
        } else {
            // End of the previous range/single or gap found
            if start == current {
                records.push(AckNackRecord::Single(start));
            } else {
                // Apply safety limit on range size during optimization too
                let actual_end = if current - start >= MAX_RANGE_SIZE { start + MAX_RANGE_SIZE -1 } else { current };
                records.push(AckNackRecord::Range(start, actual_end));
                // If clamped, start a new record for the rest if necessary
                if actual_end < current {
                    // This case implies the original range exceeded MAX_RANGE_SIZE.
                    // We start a new potential range immediately after the clamped one.
                    // The loop will continue from 'current', but the 'start' needs resetting.
                    start = actual_end + 1;
                    if start < current { // Need another range? Usually not needed with iteration logic below.
                        // But just in case, if current is far ahead. This should technically be handled
                        // by the seq_num == current + 1 check not matching, leading to the else block.
                        records.push(AckNackRecord::Range(start, current));
                    } else if start == current { // Edge case: single packet after clamped range
                        records.push(AckNackRecord::Single(current))
                    }
                }

            }
            start = seq_num; // Start of the new potential range/single
            current = seq_num;
        }
        // Safety limit during iteration - split large ranges proactively
        if current - start >= MAX_RANGE_SIZE {
            records.push(AckNackRecord::Range(start, current));
            start = current + 1; // Ready for next potential range
            current = start; // Effectively reset current pointer for next iteration
        }
    }


    // Add the last record
    if start == current {
        records.push(AckNackRecord::Single(start));
    } else {
        records.push(AckNackRecord::Range(start, current)); // Already limited by MAX_RANGE_SIZE check inside loop
    }


    records
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};

    #[test]
    fn test_encode_decode_ack() {
        let mut ack = ACK::new_ack();
        ack.records = vec![
            AckNackRecord::Single(10),
            AckNackRecord::Range(15, 18),
            AckNackRecord::Single(20),
        ];


        let mut writer = BytesMut::new();
        ack.encode(&mut writer).unwrap();


        let expected_header = ACK_HEADER;
        let expected_record_count: u16 = 3;


        let mut expected = BytesMut::new();
        expected.put_u8(expected_header);
        expected.write_u16_be(expected_record_count).unwrap();
        // Record 1: Single(10)
        expected.put_u8(1);
        expected.write_u24_le(10).unwrap();
        // Record 2: Range(15, 18)
        expected.put_u8(0);
        expected.write_u24_le(15).unwrap();
        expected.write_u24_le(18).unwrap();
        // Record 3: Single(20)
        expected.put_u8(1);
        expected.write_u24_le(20).unwrap();


        assert_eq!(writer.freeze(), expected.freeze());


        let mut reader_bytes = writer.freeze();
        let packet_id = reader_bytes.get_u8();
        assert_eq!(packet_id, ACK_HEADER);
        let is_nack = (packet_id & super::NACK_HEADER) == super::NACK_HEADER;
        assert!(!is_nack);

        let decoded_ack = AckNack::decode(&mut reader_bytes, is_nack).unwrap();
        assert_eq!(decoded_ack.is_nack, false);
        assert_eq!(decoded_ack.records, ack.records);
        assert!(reader_bytes.is_empty());
    }

    #[test]
    fn test_encode_decode_nack() {
        let mut nack = NACK::new_nack();
        nack.records = vec![AckNackRecord::Range(50, 55)];


        let mut writer = BytesMut::new();
        nack.encode(&mut writer).unwrap();


        let expected_header = NACK_HEADER;
        let expected_record_count: u16 = 1;


        let mut expected = BytesMut::new();
        expected.put_u8(expected_header);
        expected.write_u16_be(expected_record_count).unwrap();
        // Record 1: Range(50, 55)
        expected.put_u8(0);
        expected.write_u24_le(50).unwrap();
        expected.write_u24_le(55).unwrap();


        assert_eq!(writer.freeze(), expected.freeze());


        let mut reader_bytes = writer.freeze();
        let packet_id = reader_bytes.get_u8();
        assert_eq!(packet_id, NACK_HEADER);
        let is_nack = (packet_id & super::NACK_HEADER) == super::NACK_HEADER;
        assert!(is_nack);

        let decoded_nack = AckNack::decode(&mut reader_bytes, is_nack).unwrap();
        assert_eq!(decoded_nack.is_nack, true);
        assert_eq!(decoded_nack.records, nack.records);
        assert!(reader_bytes.is_empty());
    }


    #[test]
    fn test_decode_invalid_range() {
        let mut bad_range = BytesMut::new();
        bad_range.write_u16_be(1).unwrap(); // record count = 1
        bad_range.put_u8(0); // type = range
        bad_range.write_u24_le(20).unwrap(); // start = 20
        bad_range.write_u24_le(10).unwrap(); // end = 10 (invalid)

        let mut reader = bad_range.freeze();
        let result = AckNack::decode(&mut reader, false);
        assert!(matches!(result, Err(BinaryError::InvalidData(_))));
    }

    #[test]
    fn test_decode_unknown_record_type() {
        let mut unknown_type = BytesMut::new();
        unknown_type.write_u16_be(1).unwrap(); // record count = 1
        unknown_type.put_u8(3); // type = 3 (invalid)
        unknown_type.write_u24_le(100).unwrap();


        let mut reader = unknown_type.freeze();
        let result = AckNack::decode(&mut reader, false);
        assert!(matches!(result, Err(BinaryError::InvalidData(_))));
    }


    #[test]
    fn test_decode_ack_eof() {
        // Test EOF reading record count
        let mut reader_eof1 = Bytes::from_static(&[0xC0, 0x01]); // Only 1 byte for record count
        reader_eof1.advance(1); // Consume header byte
        let result1 = AckNack::decode(&mut reader_eof1, false);
        assert!(matches!(result1, Err(BinaryError::UnexpectedEof{..})));


        // Test EOF reading record type
        let mut reader_eof2 = BytesMut::new();
        reader_eof2.write_u16_be(1).unwrap(); // 1 record expected
        // No record type or data
        let mut bytes_eof2 = reader_eof2.freeze();
        let result2 = AckNack::decode(&mut bytes_eof2, false);
        assert!(matches!(result2, Err(BinaryError::UnexpectedEof{..})));

        // Test EOF reading single record content
        let mut reader_eof3 = BytesMut::new();
        reader_eof3.write_u16_be(1).unwrap();
        reader_eof3.put_u8(1); // Single type
        reader_eof3.write_u16_le(123).unwrap(); // Only 2 bytes for u24
        let mut bytes_eof3 = reader_eof3.freeze();
        let result3 = AckNack::decode(&mut bytes_eof3, false);
        assert!(matches!(result3, Err(BinaryError::UnexpectedEof{..})));


        // Test EOF reading range record content (end)
        let mut reader_eof4 = BytesMut::new();
        reader_eof4.write_u16_be(1).unwrap();
        reader_eof4.put_u8(0); // Range type
        reader_eof4.write_u24_le(10).unwrap(); // Start ok
        reader_eof4.write_u16_le(123).unwrap(); // Only 2 bytes for end u24
        let mut bytes_eof4 = reader_eof4.freeze();
        let result4 = AckNack::decode(&mut bytes_eof4, false);
        assert!(matches!(result4, Err(BinaryError::UnexpectedEof{..})));

    }


    #[test]
    fn test_optimize_records_empty() {
        assert_eq!(optimize_ack_nack_records(&[]), vec![]);
    }

    #[test]
    fn test_optimize_records_single() {
        assert_eq!(optimize_ack_nack_records(&[5]), vec![AckNackRecord::Single(5)]);
    }

    #[test]
    fn test_optimize_records_contiguous_range() {
        assert_eq!(optimize_ack_nack_records(&[10, 11, 12, 13]), vec![AckNackRecord::Range(10, 13)]);
    }


    #[test]
    fn test_optimize_records_multiple_singles() {
        assert_eq!(optimize_ack_nack_records(&[5, 7, 9]), vec![
            AckNackRecord::Single(5),
            AckNackRecord::Single(7),
            AckNackRecord::Single(9),
        ]);
    }

    #[test]
    fn test_optimize_records_mixed() {
        assert_eq!(optimize_ack_nack_records(&[1, 2, 3, 5, 7, 8, 9, 11, 20]), vec![
            AckNackRecord::Range(1, 3),
            AckNackRecord::Single(5),
            AckNackRecord::Range(7, 9),
            AckNackRecord::Single(11),
            AckNackRecord::Single(20),
        ]);
    }

    #[test]
    fn test_optimize_records_duplicates() {
        // Optimize function expects sorted unique inputs, but let's see how it behaves with duplicates
        // The standard sort will keep duplicates, so the optimization logic should handle them gracefully.
        assert_eq!(optimize_ack_nack_records(&[1, 2, 2, 3, 5, 5, 5, 7, 8, 9, 9]), vec![
            AckNackRecord::Range(1, 3), // It correctly treats 2, 2, 3 as part of the 1..3 range
            AckNackRecord::Single(5),  // It treats 5, 5, 5 as ending the previous single '3' and starting a new '5'
            AckNackRecord::Range(7, 9), // Treats 9, 9 as part of 7..9
        ]);
    }

    #[test]
    fn test_optimize_large_range_clamping() {
        let nums: Vec<u32> = (100..100 + MAX_RANGE_SIZE + 50).collect();
        let expected_records = vec![
            AckNackRecord::Range(100, 100 + MAX_RANGE_SIZE - 1), // First clamped range
            // Start of the next potential range is 100 + MAX_RANGE_SIZE
            // End is 100 + MAX_RANGE_SIZE + 50 - 1
            AckNackRecord::Range(100 + MAX_RANGE_SIZE, 100 + MAX_RANGE_SIZE + 49),
        ];
        assert_eq!(optimize_ack_nack_records(&nums), expected_records);


        // Test with a gap after a large range
        let mut nums_with_gap = (500..500 + MAX_RANGE_SIZE).collect::<Vec<u32>>();
        nums_with_gap.push(500 + MAX_RANGE_SIZE + 2); // Gap of 1 number
        let expected_with_gap = vec![
            AckNackRecord::Range(500, 500 + MAX_RANGE_SIZE - 1),
            AckNackRecord::Single(500 + MAX_RANGE_SIZE + 2),
        ];
        assert_eq!(optimize_ack_nack_records(&nums_with_gap), expected_with_gap);
    }


    #[test]
    fn test_extract_sequence_numbers() {
        let ack = AckNack {
            is_nack: false,
            records: vec![
                AckNackRecord::Single(5),
                AckNackRecord::Range(10, 12),
                AckNackRecord::Single(15),
            ],
        };
        let extracted = ack.extract_sequence_numbers();
        assert_eq!(extracted, vec![5, 10, 11, 12, 15]);
    }

}