use super::packet::PacketId;
use crate::check_remaining;
use crate::utils::{BinaryError, BinaryReader, BinaryResult, BinaryWriter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AckRecord {
    Single(u32),
    Range(u32, u32),
}

#[derive(Debug, Clone)]
pub struct AckNakPacket {
    pub is_nack: bool,
    pub records: Vec<AckRecord>,
}

impl AckNakPacket {
    pub fn id(&self) -> u8 {
        if self.is_nack {
            PacketId::NACK
        } else {
            PacketId::ACK
        }
    }

    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        let record_count = u16::try_from(self.records.len()).map_err(|_| {
            BinaryError::InvalidData("Too many ACK/NACK records to fit in u16 count".into())
        })?;
        writer.write_u16_be(record_count)?;

        for record in &self.records {
            match record {
                AckRecord::Single(seq) => {
                    writer.write_bool(true)?;
                    writer.write_u24_le(*seq)?;
                }
                AckRecord::Range(start, end) => {
                    writer.write_bool(false)?;
                    writer.write_u24_le(*start)?;
                    writer.write_u24_le(*end)?;
                }
            }
        }
        Ok(())
    }

    pub fn encode(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_u8(self.id())?;
        self.encode_payload(writer)
    }

    pub fn decode_payload(reader: &mut impl BinaryReader, is_nack: bool) -> BinaryResult<Self> {
        let record_count = reader.read_u16_be()?;
        if record_count > 8192 {
            return Err(BinaryError::InvalidData(format!(
                "Excessive ACK/NACK record count: {}",
                record_count
            )));
        }
        let mut records = Vec::with_capacity(record_count as usize);

        for _ in 0..record_count {
            check_remaining!(reader, 1);
            let is_single = reader.read_bool()?;

            if is_single {
                check_remaining!(reader, 3);
                let seq = reader.read_u24_le()?;
                records.push(AckRecord::Single(seq));
            } else {
                check_remaining!(reader, 6);
                let start = reader.read_u24_le()?;
                let end = reader.read_u24_le()?;
                if start > end {
                    return Err(BinaryError::InvalidData(format!(
                        "Invalid ACK/NACK range: start {} > end {}",
                        start, end
                    )));
                }
                records.push(AckRecord::Range(start, end));
            }
        }

        Ok(Self { is_nack, records })
    }

    pub fn decode(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        let id = reader.read_u8()?;
        let is_nack = match id {
            PacketId::ACK => false,
            PacketId::NACK => true,
            _ => {
                return Err(BinaryError::InvalidData(format!(
                    "Invalid ACK/NACK packet ID: {}",
                    id
                )));
            }
        };
        Self::decode_payload(reader, is_nack)
    }

    pub fn from_sequences(sequences: &[u32], is_nack: bool) -> Self {
        if sequences.is_empty() {
            return Self {
                is_nack,
                records: Vec::new(),
            };
        }

        let mut records = Vec::new();
        let mut iter = sequences.iter().peekable();

        while let Some(&start) = iter.next() {
            let mut end = start;
            while let Some(&&next) = iter.peek() {
                if next == end.wrapping_add(1) {
                    end = next;
                    iter.next();
                } else {
                    break;
                }
            }

            if start == end {
                records.push(AckRecord::Single(start));
            } else {
                records.push(AckRecord::Range(start, end));
            }
        }

        Self { is_nack, records }
    }
}
