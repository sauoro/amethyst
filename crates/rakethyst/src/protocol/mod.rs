use amethyst_binary::error::BinaryError;
use amethyst_binary::error::BinaryError::{InvalidData, UnexpectedEOF};
use amethyst_binary::io::{BinaryReader, BinaryWriter};
use amethyst_binary::traits::{Readable, Writable};
use bytes::{Bytes};
use std::net::SocketAddr;
use crate::connection::SequenceNumberRange;

pub const RAKNET_PROTOCOL_VERSION: u8 = 11;

pub const MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

pub const CONNECTED_PING: u8 = 0x00;
pub const UNCONNECTED_PING: u8 = 0x01;
pub const CONNECTED_PONG: u8 = 0x03;
pub const OPEN_CONNECTION_REQUEST_1: u8 = 0x05;
pub const OPEN_CONNECTION_REPLY_1: u8 = 0x06;
pub const OPEN_CONNECTION_REQUEST_2: u8 = 0x07;
pub const OPEN_CONNECTION_REPLY_2: u8 = 0x08;
pub const CONNECTION_REQUEST: u8 = 0x09;
pub const UNCONNECTED_PONG: u8 = 0x1c;
pub const CONNECTION_REQUEST_ACCEPTED: u8 = 0x10;
pub const NEW_INCOMING_CONNECTION: u8 = 0x13;
pub const ACK: u8 = 0xc0;
pub const NACK: u8 = 0xa0;

#[derive(Clone, Debug)]
pub struct ConnectedPing {
    pub time: u64,
}

impl Writable for ConnectedPing {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_u64(self.time)?;
        Ok(())
    }
}

impl Readable for ConnectedPing {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let time = reader.read_u64()?;
        Ok(Self { time })
    }
}

#[derive(Clone, Debug)]
pub struct UnconnectedPing {
    pub time: u64,
    pub client_guid: u64,
}

impl Writable for UnconnectedPing {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_u64(self.time)?;
        writer.write_bytes(MAGIC.as_slice())?;
        writer.write_u64(self.client_guid)?;
        Ok(())
    }
}

impl Readable for UnconnectedPing {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let time = reader.read_u64()?;
        let mut bytes = [0u8; 16];
        reader.read_exact(&mut bytes)?;
        if bytes != MAGIC {
            Err(InvalidData("Expected magic bytes.".to_string()))?
        }
        if reader.remaining() < 8 {
            return Err(InvalidData(
                "Packet too short for UnconnectedPing (missing client GUID)".to_string(),
            ));
        }
        let client_guid = reader.read_u64()?;
        Ok(Self { time, client_guid })
    }
}

#[derive(Clone, Debug)]
pub struct ConnectedPong {
    pub ping_time: u64,
    pub pong_time: u64,
}

impl Writable for ConnectedPong {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_u64(self.ping_time)?;
        writer.write_u64(self.pong_time)?;
        Ok(())
    }
}

impl Readable for ConnectedPong {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let ping_time = reader.read_u64()?;
        let pong_time = reader.read_u64()?;
        Ok(Self {
            ping_time,
            pong_time,
        })
    }
}

#[derive(Clone, Debug)]
pub struct UnconnectedPong {
    pub time: u64,
    pub server_guid: u64,
    pub motd: String,
}

impl Writable for UnconnectedPong {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_u64(self.time)?;
        writer.write_u64(self.server_guid)?;
        writer.write_bytes(MAGIC.as_slice())?;
        if self.motd.len() > u16::MAX as usize {
            return Err(InvalidData(format!(
                "MOTD length ({}) exceeds maximum ({})",
                self.motd.len(),
                u16::MAX
            )));
        }
        writer.write_string_u16(self.motd.as_str())?;
        Ok(())
    }
}

impl Readable for UnconnectedPong {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let time = reader.read_u64()?;
        let server_guid = reader.read_u64()?;
        let mut bytes = [0u8; 16];
        reader.read_exact(&mut bytes)?;
        if bytes != MAGIC {
            return Err(InvalidData(format!(
                "Expected magic bytes, got: {:02X?}",
                bytes
            )));
        }
        let motd = reader.read_string_u16()?;
        Ok(Self {
            time,
            server_guid,
            motd,
        })
    }
}

#[derive(Clone, Debug)]
pub struct OpenConnectionRequest1 {
    pub protocol_version: u8,
    //pub payload: Bytes,
}

impl Writable for OpenConnectionRequest1 {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_bytes(MAGIC.as_slice())?;
        writer.write_u8(self.protocol_version)?;
        //writer.write_bytes(self.payload.as_ref())?;
        Ok(())
    }
}

impl Readable for OpenConnectionRequest1 {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let mut bytes = [0u8; 16];
        reader.read_exact(&mut bytes)?;
        if bytes != MAGIC {
            return Err(InvalidData(format!(
                "Expected magic bytes, got: {:02X?}",
                bytes
            )));
        }
        let protocol_version = reader.read_u8()?;
        //let payload = reader.read_bytes(reader.remaining())?;
        Ok(Self {
            protocol_version,
            //payload,
        })
    }
}

#[derive(Clone, Debug)]
pub struct OpenConnectionReply1 {
    pub server_guid: u64,
    pub use_security: bool,
    pub mtu_size: u16,
}

impl Writable for OpenConnectionReply1 {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_bytes(MAGIC.as_slice())?;
        writer.write_u64(self.server_guid)?;
        writer.write_u8(if self.use_security { 1 } else { 0 })?;
        writer.write_u16(self.mtu_size)?;
        Ok(())
    }
}

impl Readable for OpenConnectionReply1 {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let mut bytes = [0u8; 16];
        reader.read_exact(&mut bytes)?;
        if bytes != MAGIC {
            return Err(InvalidData(format!(
                "Expected magic bytes, got: {:02X?}",
                bytes
            )));
        }
        let server_guid = reader.read_u64()?;
        let security_byte = reader.read_u8()?;
        let use_security = match security_byte {
            0 => false,
            1 => true,
            _ => {
                return Err(InvalidData(format!(
                    "Invalid value for use_security: {}",
                    security_byte
                )));
            }
        };
        let mtu_size = reader.read_u16()?;
        Ok(Self {
            server_guid,
            use_security,
            mtu_size,
        })
    }
}

#[derive(Clone, Debug)]
pub struct OpenConnectionRequest2 {
    pub server_addr: SocketAddr,
    pub mtu: u16,
    pub client_guid: u64,
}

impl Writable for OpenConnectionRequest2 {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_bytes(MAGIC.as_slice())?;
        writer.write_raknet_address(self.server_addr)?;
        writer.write_u16(self.mtu)?;
        writer.write_u64(self.client_guid)?;
        Ok(())
    }
}

impl Readable for OpenConnectionRequest2 {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let mut bytes = [0u8; 16];
        reader.read_exact(&mut bytes)?;
        if bytes != MAGIC {
            return Err(InvalidData(format!(
                "Expected magic bytes, got: {:02X?}",
                bytes
            )));
        }
        let server_addr = reader.read_raknet_address()?;
        let mtu = reader.read_u16()?;
        let client_guid = reader.read_u64()?;
        Ok(Self {
            server_addr,
            mtu,
            client_guid,
        })
    }
}

#[derive(Clone, Debug)]
pub struct OpenConnectionReply2 {
    pub server_guid: u64,
    pub client_addr: SocketAddr,
    pub mtu: u16,
    pub use_encryption: bool,
}

impl Writable for OpenConnectionReply2 {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_bytes(MAGIC.as_slice())?;
        writer.write_u64(self.server_guid)?;
        writer.write_raknet_address(self.client_addr)?;
        writer.write_u16(self.mtu)?;
        writer.write_u8(if self.use_encryption { 1 } else { 0 })?;
        Ok(())
    }
}

impl Readable for OpenConnectionReply2 {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let mut bytes = [0u8; 16];
        reader.read_exact(&mut bytes)?;
        if bytes != MAGIC {
            return Err(InvalidData(format!(
                "Expected magic bytes, got: {:02X?}",
                bytes
            )));
        }
        let server_guid = reader.read_u64()?;
        let client_addr = reader.read_raknet_address()?;
        let mtu = reader.read_u16()?;
        let encryption_byte = reader.read_u8()?;
        let use_encryption = match encryption_byte {
            0 => false,
            1 => true,
            _ => {
                return Err(InvalidData(format!(
                    "Invalid value for use_encryption: {}",
                    encryption_byte
                )));
            }
        };
        Ok(Self {
            server_guid,
            client_addr,
            mtu,
            use_encryption,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionRequest {
    pub client_guid: u64,
    pub time: u64,
    pub use_security: bool,
}

impl Writable for ConnectionRequest {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_u64(self.client_guid)?;
        writer.write_u64(self.time)?;
        writer.write_u8(if self.use_security { 1 } else { 0 })?;
        Ok(())
    }
}

impl Readable for ConnectionRequest {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let client_guid = reader.read_u64()?;
        let time = reader.read_u64()?;
        let security_byte = reader.read_u8()?;
        let use_security = match security_byte {
            0 => false,
            1 => true,
            _ => {
                return Err(InvalidData(format!(
                    "Invalid value for use_security: {}",
                    security_byte
                )));
            }
        };
        Ok(Self {
            client_guid,
            time,
            use_security,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionRequestAccepted {
    pub client_address: SocketAddr,
    pub system_index: u16,
    pub internal_ids: [SocketAddr; 20],
    pub request_time: u64,
    pub time: u64,
}

impl Writable for ConnectionRequestAccepted {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_socket_addr(&self.client_address)?;
        writer.write_u16(self.system_index)?;
        for addr in &self.internal_ids {
            writer.write_socket_addr(addr)?;
        }
        writer.write_u64(self.request_time)?;
        writer.write_u64(self.time)?;
        Ok(())
    }
}

impl Readable for ConnectionRequestAccepted {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let client_address = reader.read_socket_addr()?;
        let system_index = reader.read_u16()?;
        let mut internal_ids =
            [SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0); 20];
        for addr in internal_ids.iter_mut() {
            *addr = reader.read_socket_addr()?;
        }
        let request_time = reader.read_u64()?;
        let time = reader.read_u64()?;
        Ok(Self {
            client_address,
            system_index,
            internal_ids,
            request_time,
            time,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AckNackRecord {
    Single(u32),
    Range(SequenceNumberRange),
}

impl Readable for AckNackRecord {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let is_range = reader.read_u8()? != 0;
        if is_range {
            let start = reader.read_u24_le()?;
            let end = reader.read_u24_le()?;
            if start > end {
                return Err(InvalidData(format!(
                    "Invalid ACK/NACK range: start ({}) > end ({})",
                    start, end
                )));
            }
            Ok(AckNackRecord::Range(SequenceNumberRange { start, end }))
        } else {
            let seq_num = reader.read_u24_le()?;
            Ok(AckNackRecord::Single(seq_num))
        }
    }
}

impl Writable for AckNackRecord {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        match self {
            AckNackRecord::Range(range) => {
                writer.write_u8(1)?;
                writer.write_u24_le(range.start)?;
                writer.write_u24_le(range.end)?;
            }
            AckNackRecord::Single(seq_num) => {
                writer.write_u8(0)?;
                writer.write_u24_le(*seq_num)?;
            }
        }
        Ok(())
    }
}


#[derive(Debug, Clone, Default)]
pub struct AckNackPacket {
    pub records: Vec<AckNackRecord>,
}

impl Readable for AckNackPacket {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let record_count = reader.read_u16()?;
        let mut records = Vec::with_capacity(record_count as usize);
        for _ in 0..record_count {
            records.push(AckNackRecord::read(reader)?);
        }
        Ok(AckNackPacket { records })
    }
}

impl Writable for AckNackPacket {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        if self.records.len() > u16::MAX as usize {
            return Err(InvalidData(format!(
                "Too many ACK/NACK records: {}",
                self.records.len()
            )));
        }
        writer.write_u16(self.records.len() as u16)?;
        for record in &self.records {
            record.write(writer)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reliability {
    Unreliable = 0,
    UnreliableSequenced = 1,
    Reliable = 2,
    ReliableOrdered = 3,
    ReliableSequenced = 4,
    UnreliableWithAckReceipt = 5,
    ReliableWithAckReceipt = 6,
    ReliableOrderedWithAckReceipt = 7,
}

impl Reliability {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Reliability::Unreliable),
            1 => Some(Reliability::UnreliableSequenced),
            2 => Some(Reliability::Reliable),
            3 => Some(Reliability::ReliableOrdered),
            4 => Some(Reliability::ReliableSequenced),
            5 => Some(Reliability::UnreliableWithAckReceipt),
            6 => Some(Reliability::ReliableWithAckReceipt),
            7 => Some(Reliability::ReliableOrderedWithAckReceipt),
            _ => None,
        }
    }

    pub fn is_reliable(&self) -> bool {
        matches!(self, Reliability::Reliable | Reliability::ReliableOrdered | Reliability::ReliableSequenced | Reliability::ReliableWithAckReceipt | Reliability::ReliableOrderedWithAckReceipt)
    }

    pub fn is_ordered(&self) -> bool {
        matches!(self, Reliability::UnreliableSequenced | Reliability::ReliableOrdered | Reliability::ReliableSequenced | Reliability::ReliableOrderedWithAckReceipt)
    }

    pub fn is_sequenced(&self) -> bool {
        self.is_ordered()
    }
}

#[derive(Debug, Clone)]
pub struct EncapsulatedPacket {
    pub reliability: Reliability,
    pub is_split: bool,
    pub sequence_number: Option<u32>,
    pub ordering_index: Option<u32>,
    pub ordering_channel: Option<u8>,
    pub split_count: Option<u32>,
    pub split_id: Option<u16>,
    pub split_index: Option<u32>,
    pub payload: Bytes,
}

impl Readable for EncapsulatedPacket {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let flags = reader.read_u8()?;
        let reliability_val = (flags >> 5) & 0x07;
        let reliability = Reliability::from_u8(reliability_val)
            .ok_or_else(|| InvalidData(format!("Invalid reliability value: {}", reliability_val)))?;
        let is_split = (flags & 0x10) != 0;

        let payload_len_bits = reader.read_u16()? as usize;
        let payload_len_bytes = (payload_len_bits + 7) / 8;

        let mut sequence_number: Option<u32> = None;
        let mut ordering_index: Option<u32> = None;
        let mut ordering_channel: Option<u8> = None;

        if reliability.is_reliable() {
            sequence_number = Some(reader.read_u24_le()?);
        }

        if reliability.is_sequenced() {
            ordering_index = Some(reader.read_u24_le()?);
            ordering_channel = Some(reader.read_u8()?);
        }

        let mut split_count: Option<u32> = None;
        let mut split_id: Option<u16> = None;
        let mut split_index: Option<u32> = None;

        if is_split {
            split_count = Some(reader.read_u32()?); // BE
            split_id = Some(reader.read_u16()?); // BE
            split_index = Some(reader.read_u32()?); // BE
        }

        if reader.remaining() < payload_len_bytes {
            return Err(UnexpectedEOF);
        }
        let payload = reader.read_bytes(payload_len_bytes)?;

        Ok(Self {
            reliability,
            is_split,
            sequence_number,
            ordering_index,
            ordering_channel,
            split_count,
            split_id,
            split_index,
            payload,
        })
    }
}

impl Writable for EncapsulatedPacket {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        let mut flags = (self.reliability as u8) << 5;
        if self.is_split {
            flags |= 0x10;
            return Err(InvalidData("Sending split packets not implemented".to_string()));
        }
        writer.write_u8(flags)?;

        let payload_len_bits = (self.payload.len() * 8) as u16;
        writer.write_u16(payload_len_bits)?;

        if self.reliability.is_reliable() {
            writer.write_u24_le(self.sequence_number.ok_or_else(|| {
                InvalidData("Reliable packet missing sequence number for writing".to_string())
            })?)?;
        }

        if self.reliability.is_sequenced() {
            writer.write_u24_le(self.ordering_index.ok_or_else(|| {
                InvalidData("Ordered/Sequenced packet missing ordering index for writing".to_string())
            })?)?;
            writer.write_u8(self.ordering_channel.ok_or_else(|| {
                InvalidData("Ordered/Sequenced packet missing ordering channel for writing".to_string())
            })?)?;
        }

        if self.is_split {
            return Err(InvalidData("Cannot write split packet header (NYI)".to_string()));
        }

        writer.write_bytes(&self.payload)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FrameSetPacket {
    pub sequence_number: u32,
    pub packets: Vec<EncapsulatedPacket>,
}

impl Readable for FrameSetPacket {
    fn read(reader: &mut BinaryReader) -> Result<Self, BinaryError> {
        let sequence_number = reader.read_u24_le()?;
        let mut packets = Vec::new();
        while reader.remaining() > 0 {
            match EncapsulatedPacket::read(reader) {
                Ok(packet) => packets.push(packet),
                Err(_e @ UnexpectedEOF) => {
                    log::trace!("EOF encountered reading encapsulated packet, assuming end of frame. Remaining: {}", reader.remaining());
                    break;
                }
                Err(e) => {
                    log::error!("Error parsing encapsulated packet: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(Self {
            sequence_number,
            packets,
        })
    }
}

impl Writable for FrameSetPacket {
    fn write(&self, writer: &mut BinaryWriter) -> Result<(), BinaryError> {
        writer.write_u24_le(self.sequence_number)?;
        for packet in &self.packets {
            packet.write(writer)?;
        }
        Ok(())
    }
}
