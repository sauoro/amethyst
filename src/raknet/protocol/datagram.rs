// src/raknet/protocol/datagram.rs
//! Structures related to RakNet datagrams and encapsulated packets.

use crate::utils::binary::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};

// --- Constants ---
pub const FLAG_VALID: u8 = 0x80;
pub const FLAG_ACK: u8 = 0x40;
pub const FLAG_NACK: u8 = 0x20;
// --- Not directly used by amethyst currently but part of protocol spec ---
// pub const FLAG_PACKET_PAIR: u8 = 0x10; // Deprecated or unused in modern RakNet?
// pub const FLAG_CONTINUOUS_SEND: u8 = 0x08; // Internal flag, seems related to split packets needing immediate followup
pub const FLAG_NEEDS_B_AND_AS: u8 = 0x04; // Internal flag related to bandwidth/ack management, MC uses it often


// --- Packet Structures ---

/// Represents a RakNet datagram containing one or more EncapsulatedPackets.
#[derive(Debug, Clone)]
pub struct Datagram {
    /// Combination of flags (FLAG_VALID, FLAG_ACK, FLAG_NACK, etc.).
    pub header_flags: u8,
    /// Sequence number for this datagram (used for ACK/NACK).
    pub sequence_number: u32, // Stored as LE Triad (u24)
    /// The encapsulated packets carried within this datagram.
    pub packets: Vec<EncapsulatedPacket>,

    // --- Metadata (not part of encoded packet) ---
    /// Time the datagram was sent (for RTT calculation). Set by sender.
    pub send_time: Option<std::time::Instant>,
    /// Next allowed send time based on congestion control. Set by sender.
    pub next_send_time: Option<std::time::Instant>,
    /// Size of the datagram in bytes for congestion control calculation. Set by sender.
    pub size_for_congestion_control: Option<usize>
}

impl Datagram {
    /// Creates a new, empty Datagram.
    pub fn new(sequence_number: u32) -> Self {
        Self {
            header_flags: FLAG_VALID | FLAG_NEEDS_B_AND_AS, // Default flags for data datagrams
            sequence_number,
            packets: Vec::new(),
            send_time: None,
            next_send_time: None,
            size_for_congestion_control: None
        }
    }

    /// Decodes a Datagram from a byte slice.
    pub fn decode(reader: &mut Bytes) -> Result<Self, BinaryError> {
        if reader.remaining() < 4 { // Min size: flags(1) + seq_num(3)
            return Err(BinaryError::UnexpectedEof { needed: 4, remaining: reader.remaining() });
        }
        let header_flags = reader.get_u8();
        if (header_flags & FLAG_VALID) == 0 {
            return Err(BinaryError::InvalidData("Invalid datagram: VALID flag not set".into()));
        }

        let sequence_number = reader.read_u24_le()?;
        let mut packets = Vec::new();

        while reader.has_remaining() {
            packets.push(EncapsulatedPacket::decode(reader)?);
        }

        Ok(Self {
            header_flags,
            sequence_number,
            packets,
            send_time: None,
            next_send_time: None,
            size_for_congestion_control: None
        })
    }

    /// Encodes the Datagram into a BytesMut buffer.
    pub fn encode(&self, writer: &mut BytesMut) -> Result<(), BinaryError> {
        writer.put_u8(self.header_flags);
        writer.write_u24_le(self.sequence_number)?;

        for packet in &self.packets {
            packet.encode(writer)?;
        }
        Ok(())
    }

    /// Calculates the encoded size of the datagram, including headers and all encapsulated packets.
    pub fn calculate_size(&self) -> usize {
        let mut size = 1 + 3; // Flags (1) + Sequence Number (3)
        for packet in &self.packets {
            size += packet.calculate_size();
        }
        size
    }
}

/// Represents a single packet encapsulated within a RakNet Datagram.
#[derive(Debug, Clone)]
pub struct EncapsulatedPacket {
    pub reliability: Reliability,
    pub is_split: bool,
    // --- Reliability dependent fields ---
    /// Reliable message number (only for reliable variants).
    pub message_index: Option<u32>, // Stored as LE Triad (u24)
    /// Sequence number (only for sequenced variants).
    pub sequence_index: Option<u32>, // Stored as LE Triad (u24)
    /// Ordering index (only for ordered/sequenced variants).
    pub order_index: Option<u32>, // Stored as LE Triad (u24)
    /// Ordering channel (only for ordered/sequenced variants).
    pub order_channel: Option<u8>,
    // --- Split packet fields (only if is_split is true) ---
    /// Number of fragments this packet is split into.
    pub split_count: Option<u32>, // u32 BE
    /// ID shared among all fragments of the same split packet.
    pub split_id: Option<u16>, // u16 BE
    /// Index of this fragment (0-based).
    pub split_index: Option<u32>, // u32 BE
    // --- The actual payload ---
    pub buffer: Bytes,

    // --- Metadata ---
    /// ACK identifier for tracking send receipts. Not encoded.
    pub ack_id: Option<u32>
}

impl EncapsulatedPacket {
    /// Decodes an EncapsulatedPacket from a buffer.
    pub fn decode(reader: &mut Bytes) -> Result<Self, BinaryError> {
        if reader.remaining() < 3 { // flags (1) + len (2)
            return Err(BinaryError::UnexpectedEof { needed: 3, remaining: reader.remaining() });
        }
        let flags = reader.get_u8();
        let reliability = Reliability::from_u8(flags >> 5)?; // Top 3 bits
        let is_split = (flags & 0x10) != 0;
        let _needs_b_and_as = (flags & 0x04) != 0; // Seems less used internally now?


        let length_bits = reader.read_u16_be()?;
        let length_bytes = ((length_bits + 7) / 8) as usize; // Round up to nearest byte

        let mut message_index = None;
        let mut sequence_index = None;
        let mut order_index = None;
        let mut order_channel = None;

        if reliability.is_reliable() {
            message_index = Some(reader.read_u24_le()?);
        }
        if reliability.is_sequenced() {
            sequence_index = Some(reader.read_u24_le()?);
        }
        if reliability.is_ordered() || reliability.is_sequenced() {
            order_index = Some(reader.read_u24_le()?);
            order_channel = Some(reader.read_u8()?);
        }


        let mut split_count = None;
        let mut split_id = None;
        let mut split_index = None;

        if is_split {
            split_count = Some(reader.read_u32_be()?);
            split_id = Some(reader.read_u16_be()?);
            split_index = Some(reader.read_u32_be()?);
        }


        if reader.remaining() < length_bytes {
            return Err(BinaryError::UnexpectedEof { needed: length_bytes, remaining: reader.remaining() });
        }
        let buffer = reader.copy_to_bytes(length_bytes);

        Ok(Self {
            reliability,
            is_split,
            message_index,
            sequence_index,
            order_index,
            order_channel,
            split_count,
            split_id,
            split_index,
            buffer,
            ack_id: None, // Not encoded
        })
    }

    /// Encodes the EncapsulatedPacket into a buffer.
    pub fn encode(&self, writer: &mut BytesMut) -> Result<(), BinaryError> {
        let mut flags = (self.reliability as u8) << 5;
        if self.is_split {
            flags |= 0x10;
        }
        // Note: FLAG_NEEDS_B_AND_AS might be needed by Minecraft, but adding it unconditionally
        // might not be correct for all RakNet usage. Add if required by testing/protocol.
        // flags |= FLAG_NEEDS_B_AND_AS;


        writer.put_u8(flags);

        let length_bits = self.buffer.len() * 8;
        if length_bits > u16::MAX as usize {
            // Should realistically not happen with MTU splitting, but check anyway.
            return Err(BinaryError::InvalidData(format!(
                "Encapsulated packet buffer too large: {} bytes",
                self.buffer.len()
            )));
        }
        writer.write_u16_be(length_bits as u16)?;

        if self.reliability.is_reliable() {
            writer.write_u24_le(self.message_index.ok_or(BinaryError::InvalidData(
                "Missing message_index for reliable packet".to_string(),
            ))?)?;
        }

        if self.reliability.is_sequenced() {
            writer.write_u24_le(self.sequence_index.ok_or(BinaryError::InvalidData(
                "Missing sequence_index for sequenced packet".to_string(),
            ))?)?;
        }

        if self.reliability.is_ordered() || self.reliability.is_sequenced() {
            writer.write_u24_le(self.order_index.ok_or(BinaryError::InvalidData(
                "Missing order_index for ordered/sequenced packet".to_string(),
            ))?)?;
            writer.write_u8(self.order_channel.ok_or(BinaryError::InvalidData(
                "Missing order_channel for ordered/sequenced packet".to_string(),
            ))?)?;
        }


        if self.is_split {
            writer.write_u32_be(self.split_count.ok_or(BinaryError::InvalidData(
                "Missing split_count for split packet".to_string(),
            ))?)?;
            writer.write_u16_be(self.split_id.ok_or(BinaryError::InvalidData(
                "Missing split_id for split packet".to_string(),
            ))?)?;
            writer.write_u32_be(self.split_index.ok_or(BinaryError::InvalidData(
                "Missing split_index for split packet".to_string(),
            ))?)?;
        }


        writer.put(self.buffer.clone()); // Clone Bytes for encoding

        Ok(())
    }

    /// Calculates the header size of the encapsulated packet based on its reliability and split status.
    pub fn header_size(&self) -> usize {
        let mut size = 1 + 2; // Flags (1) + Length (2)
        if self.reliability.is_reliable() {
            size += 3; // message_index
        }
        if self.reliability.is_sequenced() {
            size += 3; // sequence_index
        }
        if self.reliability.is_ordered() || self.reliability.is_sequenced() {
            size += 3 + 1; // order_index + order_channel
        }
        if self.is_split {
            size += 4 + 2 + 4; // split_count + split_id + split_index
        }
        size
    }


    /// Calculates the total size of the encoded encapsulated packet (header + buffer).
    pub fn calculate_size(&self) -> usize {
        self.header_size() + self.buffer.len()
    }

    /// Creates a simple EncapsulatedPacket with default reliability (ReliableOrdered) and no splitting.
    pub fn simple(buffer: Bytes, message_index: Option<u32>, order_index: Option<u32>) -> Self {
        Self {
            reliability: Reliability::ReliableOrdered, // Common default for game packets
            is_split: false,
            message_index,
            sequence_index: None,
            order_index,
            order_channel: Some(0), // Default channel 0
            split_count: None,
            split_id: None,
            split_index: None,
            buffer,
            ack_id: None
        }
    }

}

/// Enum representing RakNet packet reliability types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Reliability {
    Unreliable = 0,
    UnreliableSequenced = 1,
    Reliable = 2,
    ReliableOrdered = 3,
    ReliableSequenced = 4,
    UnreliableWithAckReceipt = 5, // Often mapped to Unreliable? Need to confirm MCBE usage
    ReliableWithAckReceipt = 6,   // Often mapped to Reliable?
    ReliableOrderedWithAckReceipt = 7, // Often mapped to ReliableOrdered?
}

impl Reliability {
    pub fn from_u8(val: u8) -> Result<Self, BinaryError> {
        Self::try_from(val).map_err(|_| BinaryError::InvalidData(format!("Invalid reliability ID: {}", val)))
    }

    /// Returns `true` if the reliability type requires a message index (Reliable*).
    #[inline]
    pub fn is_reliable(self) -> bool {
        matches!(
            self,
            Reliability::Reliable
                | Reliability::ReliableOrdered
                | Reliability::ReliableSequenced
                | Reliability::ReliableWithAckReceipt
                | Reliability::ReliableOrderedWithAckReceipt
                | Reliability::ReliableOrdered // Added missing variant that should be reliable
                // Consider ReliableSequencedWithAckReceipt if added later
        )
    }

    /// Returns `true` if the reliability type uses ordering (ReliableOrdered*).
    #[inline]
    pub fn is_ordered(self) -> bool {
        matches!(self, Reliability::ReliableOrdered | Reliability::ReliableOrderedWithAckReceipt)
    }

    /// Returns `true` if the reliability type uses sequencing (*Sequenced).
    #[inline]
    pub fn is_sequenced(self) -> bool {
        matches!(self, Reliability::UnreliableSequenced | Reliability::ReliableSequenced)
        // Should include ReliableSequencedWithAckReceipt if that's ever defined/used
    }

    /// Returns `true` if the packet needs an ordering channel and index.
    #[inline]
    pub fn needs_ordering_info(self) -> bool {
        self.is_ordered() || self.is_sequenced()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::{BinaryReader, BinaryWritter};
    use bytes::BytesMut;


    #[test]
    fn test_encode_decode_simple_encapsulated() {
        let payload = Bytes::from_static(&[0xFE, 0x01, 0x02, 0x03]);
        let packet = EncapsulatedPacket {
            reliability: Reliability::ReliableOrdered,
            is_split: false,
            message_index: Some(10),
            sequence_index: None, // Not needed for ReliableOrdered
            order_index: Some(5),
            order_channel: Some(0),
            split_count: None,
            split_id: None,
            split_index: None,
            buffer: payload.clone(),
            ack_id: None,
        };


        let mut writer = BytesMut::new();
        packet.encode(&mut writer).unwrap();


        // Expected: Flags (ReliableOrdered=3 << 5 = 0x60) | Len (4*8=32 -> 0x0020 BE) | MsgIdx (10 LE -> 0x0A 0x00 0x00) | OrdIdx (5 LE -> 0x05 0x00 0x00) | OrdCh (0) | Payload
        let expected = Bytes::from_static(&[
            0x60, // Flags (reliability 3 << 5)
            0x00, 0x20, // Length (32 bits = 4 bytes) BE
            0x0A, 0x00, 0x00, // Message Index (10) LE Triad
            0x05, 0x00, 0x00, // Order Index (5) LE Triad
            0x00, // Order Channel (0)
            0xFE, 0x01, 0x02, 0x03, // Payload
        ]);


        assert_eq!(writer.freeze(), expected);


        let mut reader_bytes = expected; // Use the expected Bytes directly
        let decoded_packet = EncapsulatedPacket::decode(&mut reader_bytes).unwrap();


        assert_eq!(decoded_packet.reliability, Reliability::ReliableOrdered);
        assert!(!decoded_packet.is_split);
        assert_eq!(decoded_packet.message_index, Some(10));
        assert_eq!(decoded_packet.order_index, Some(5));
        assert_eq!(decoded_packet.order_channel, Some(0));
        assert_eq!(decoded_packet.buffer, payload);
        assert!(reader_bytes.is_empty());
    }


    #[test]
    fn test_encode_decode_split_encapsulated() {
        let payload = Bytes::from_static(&[0xAA, 0xBB, 0xCC]);
        let packet = EncapsulatedPacket {
            reliability: Reliability::Reliable,
            is_split: true,
            message_index: Some(20),
            sequence_index: None, // Not needed for Reliable
            order_index: None, // Not needed for Reliable
            order_channel: None, // Not needed for Reliable
            split_count: Some(2),
            split_id: Some(1234), // 0x04D2
            split_index: Some(0),
            buffer: payload.clone(),
            ack_id: None,
        };


        let mut writer = BytesMut::new();
        packet.encode(&mut writer).unwrap();


        // Expected: Flags ((Rel=2<<5)|Split=0x10 => 0x40|0x10=0x50) | Len (3*8=24 -> 0x0018 BE) | MsgIdx (20 LE->0x14 00 00) | SplitCount (2 BE) | SplitId (1234 BE -> 0x04 D2) | SplitIdx (0 BE) | Payload
        let expected = Bytes::from_static(&[
            0x50, // Flags (reliability 2 << 5 | split 0x10)
            0x00, 0x18, // Length (24 bits = 3 bytes) BE
            0x14, 0x00, 0x00, // Message Index (20) LE Triad
            0x00, 0x00, 0x00, 0x02, // Split Count (2) BE u32
            0x04, 0xD2, // Split ID (1234) BE u16
            0x00, 0x00, 0x00, 0x00, // Split Index (0) BE u32
            0xAA, 0xBB, 0xCC, // Payload
        ]);


        assert_eq!(writer.freeze(), expected);


        let mut reader_bytes = expected;
        let decoded_packet = EncapsulatedPacket::decode(&mut reader_bytes).unwrap();


        assert_eq!(decoded_packet.reliability, Reliability::Reliable);
        assert!(decoded_packet.is_split);
        assert_eq!(decoded_packet.message_index, Some(20));
        assert_eq!(decoded_packet.split_count, Some(2));
        assert_eq!(decoded_packet.split_id, Some(1234));
        assert_eq!(decoded_packet.split_index, Some(0));
        assert_eq!(decoded_packet.buffer, payload);
        assert!(reader_bytes.is_empty());
    }


    #[test]
    fn test_encapsulated_header_size() {
        let p1 = EncapsulatedPacket::simple(Bytes::new(), Some(1), Some(1)); // ReliableOrdered
        assert_eq!(p1.header_size(), 1 + 2 + 3 + 3 + 1); // flags+len+msgidx+ordidx+ordch

        let p2 = EncapsulatedPacket { reliability: Reliability::Unreliable, is_split: false, ..p1 };
        assert_eq!(p2.header_size(), 1 + 2); // flags+len

        let p3 = EncapsulatedPacket { reliability: Reliability::Reliable, is_split: true, message_index: Some(1), split_count: Some(1), split_id: Some(1), split_index: Some(1), ..p1 };
        assert_eq!(p3.header_size(), 1 + 2 + 3 + 10); // flags+len+msgidx + split_info

        let p4 = EncapsulatedPacket { reliability: Reliability::UnreliableSequenced, is_split: false, sequence_index: Some(1), order_index: Some(1), order_channel: Some(0), ..p1 };
        assert_eq!(p4.header_size(), 1 + 2 + 3 + 3 + 1); // flags+len+seqidx+ordidx+ordch
    }


    #[test]
    fn test_decode_datagram() {
        let payload1 = Bytes::from_static(&[0xFE, 0x01, 0x02]);
        let payload2 = Bytes::from_static(&[0xFE, 0xAA]);
        let mut enc1_writer = BytesMut::new();
        EncapsulatedPacket { reliability: Reliability::Unreliable, is_split: false, buffer: payload1.clone(), message_index:None, sequence_index: None, order_index: None, order_channel: None, split_count: None, split_id: None, split_index: None, ack_id: None }.encode(&mut enc1_writer).unwrap();
        let mut enc2_writer = BytesMut::new();
        EncapsulatedPacket::simple(payload2.clone(), Some(5), Some(10)).encode(&mut enc2_writer).unwrap();


        let mut writer = BytesMut::new();
        writer.put_u8(FLAG_VALID | FLAG_NEEDS_B_AND_AS); // Header Flags
        writer.write_u24_le(12345).unwrap(); // Sequence number
        writer.put(enc1_writer.freeze());
        writer.put(enc2_writer.freeze());


        let mut reader_bytes = writer.freeze();
        let datagram = Datagram::decode(&mut reader_bytes).unwrap();


        assert_eq!(datagram.header_flags, FLAG_VALID | FLAG_NEEDS_B_AND_AS);
        assert_eq!(datagram.sequence_number, 12345);
        assert_eq!(datagram.packets.len(), 2);


        assert_eq!(datagram.packets[0].reliability, Reliability::Unreliable);
        assert!(!datagram.packets[0].is_split);
        assert_eq!(datagram.packets[0].buffer, payload1);


        assert_eq!(datagram.packets[1].reliability, Reliability::ReliableOrdered);
        assert!(!datagram.packets[1].is_split);
        assert_eq!(datagram.packets[1].message_index, Some(5));
        assert_eq!(datagram.packets[1].order_index, Some(10));
        assert_eq!(datagram.packets[1].buffer, payload2);

        assert!(reader_bytes.is_empty());

    }


    #[test]
    fn test_decode_datagram_eof() {
        let mut reader_bytes = Bytes::from_static(&[FLAG_VALID, 0x01, 0x00]); // Too short for sequence number
        let result = Datagram::decode(&mut reader_bytes);
        assert!(matches!(result, Err(BinaryError::UnexpectedEof { needed: 4, .. })));

        let mut writer = BytesMut::new();
        writer.put_u8(FLAG_VALID);
        writer.write_u24_le(1).unwrap();
        // Write partial encapsulated packet header
        writer.put_u8(0x00); // Reliability=0, no split
        writer.write_u16_be(8*5).unwrap(); // length 5 bytes
        writer.put(&[0x01, 0x02, 0x03][..]); // Only 3 bytes of payload
        let mut reader_payload_eof = writer.freeze();
        let result_payload = Datagram::decode(&mut reader_payload_eof);
        assert!(matches!(result_payload, Err(BinaryError::UnexpectedEof { needed: 5, .. })));
    }
}