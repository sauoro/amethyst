use super::{reliability::Reliability, RaknetError, Result as RaknetResult};
use crate::check_remaining;
use crate::utils::{BinaryError, BinaryReader, BinaryResult, BinaryWriter};
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct FragmentInfo {
    pub count: u32,
    pub id: u16,
    pub index: u32,
}

impl FragmentInfo {
    pub fn encode(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_u32_be(self.count)?;
        writer.write_u16_be(self.id)?;
        writer.write_u32_be(self.index)?;
        Ok(())
    }

    pub fn decode(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        // Ensure enough bytes for fragment info
        if reader.remaining() < 10 {
            return Err(BinaryError::UnexpectedEof {
                needed: 10,
                remaining: reader.remaining(),
            });
        }
        let count = reader.read_u32_be()?;
        let id = reader.read_u16_be()?;
        let index = reader.read_u32_be()?;
        Ok(Self { count, id, index })
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub reliability: Reliability,
    pub fragmented: bool,
    pub reliable_frame_index: Option<u32>,
    pub sequenced_frame_index: Option<u32>,
    pub order_frame_index: Option<u32>,
    pub order_channel: Option<u8>,
    pub fragment_info: Option<FragmentInfo>,
    pub body: Bytes,
}
impl Frame {
    pub fn size_hint(&self) -> usize {
        let mut size = 1 + 2;
        if self.reliability.is_reliable() {
            size += 3;
        }
        if self.reliability.is_sequenced() {
            size += 3;
        }
        if self.reliability.is_ordered() {
            size += 4;
        }
        if self.fragmented {
            size += 10;
        }
        size += self.body.len();
        size
    }

    pub fn encode(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        let mut flags: u8 = (self.reliability as u8) << 5;
        if self.fragmented {
            flags |= 0b0001_0000;
        }
        writer.write_u8(flags)?;

        let body_len_bits = self
            .body
            .len()
            .checked_mul(8)
            .and_then(|l| u16::try_from(l).ok())
            .ok_or_else(|| {
                BinaryError::InvalidData("Frame body too large to encode length in u16 bits".into())
            })?;
        writer.write_u16_be(body_len_bits)?;

        if let Some(index) = self.reliable_frame_index {
            writer.write_u24_le(index)?;
        }
        if let Some(index) = self.sequenced_frame_index {
            writer.write_u24_le(index)?;
        }
        if let Some(index) = self.order_frame_index {
            writer.write_u24_le(index)?;
            writer.write_u8(self.order_channel.unwrap_or(0))?;
        }
        if let Some(ref fragment_info) = self.fragment_info {
            fragment_info.encode(writer)?;
        }

        writer.write_bytes(&self.body)?;
        Ok(())
    }

    pub fn decode(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        check_remaining!(reader, 1);
        let flags = reader.read_u8()?;
        let reliability_val = (flags & 0b1110_0000) >> 5;
        let fragmented = (flags & 0b0001_0000) != 0;

        let reliability = Reliability::from_u8(reliability_val).ok_or_else(|| {
            BinaryError::InvalidData(format!("Invalid reliability value {}", reliability_val))
        })?;

        check_remaining!(reader, 2);
        let body_len_bits = reader.read_u16_be()?;
        let body_len_bytes = (body_len_bits as usize + 7) / 8;

        let reliable_frame_index = if reliability.is_reliable() {
            check_remaining!(reader, 3);
            Some(reader.read_u24_le()?)
        } else {
            None
        };

        let sequenced_frame_index = if reliability.is_sequenced() {
            check_remaining!(reader, 3);
            Some(reader.read_u24_le()?)
        } else {
            None
        };

        let (order_frame_index, order_channel) = if reliability.is_ordered() {
            check_remaining!(reader, 4); // u24 + u8
            (Some(reader.read_u24_le()?), Some(reader.read_u8()?))
        } else {
            (None, None)
        };

        let fragment_info = if fragmented {
            Some(FragmentInfo::decode(reader)?)
        } else {
            None
        };

        let body = reader.read_bytes(body_len_bytes)?;

        Ok(Self {
            reliability,
            fragmented,
            reliable_frame_index,
            sequenced_frame_index,
            order_frame_index,
            order_channel,
            fragment_info,
            body,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FramePacket {
    pub sequence_number: u32,
    pub frames: Vec<Frame>,
}

impl FramePacket {
    pub fn size_hint(&self) -> usize {
        let mut size = 1 + 3;
        for frame in &self.frames {
            size += frame.size_hint();
        }
        size
    }

    pub fn encode(&self, writer: &mut impl BinaryWriter) -> RaknetResult<()> {
        writer.write_u8(0x84)?;
        writer.write_u24_le(self.sequence_number)?;

        for frame in &self.frames {
            frame.encode(writer)?;
        }
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> RaknetResult<Self> {
        let sequence_number = reader.read_u24_le()?;

        let mut frames = Vec::new();
        while reader.has_remaining() {
            match Frame::decode(reader) {
                Ok(frame) => frames.push(frame),
                Err(BinaryError::UnexpectedEof { .. }) => {
                    break;
                }
                Err(e) => {
                    return Err(RaknetError::BinaryError(e));
                }
            }
        }

        Ok(Self {
            sequence_number,
            frames,
        })
    }

    pub fn decode_with_id(reader: &mut impl BinaryReader) -> RaknetResult<Self> {
        let id = reader.read_u8()?;
        if !(0x80..=0x8d).contains(&id) {
            return Err(RaknetError::InvalidPacketId(id));
        }
        Self::decode_payload(reader)
    }
}
