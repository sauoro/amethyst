use crate::packet::implement_packet;
use crate::packet::PacketId;
use binary::{BinaryError, BinaryReader, BinaryResult, BinaryWriter};
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct OpenConnectionRequest1 {
    pub protocol_version: u8,
}

impl OpenConnectionRequest1 {
    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_magic()?;
        writer.write_u8(self.protocol_version)?;
        let current_len = 1 + 16 + 1;
        let padding_len = crate::consts::DEFAULT_MTU.saturating_sub(current_len as u16 + 28);
        if padding_len > 0 {
            writer.write_bytes(&vec![0; padding_len as usize])?;
        }
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        if !reader.read_magic()? {
            return Err(binary::BinaryError::InvalidData(
                "Invalid magic sequence in OpenConnectionRequest1".into(),
            ));
        }
        let protocol_version = reader.read_u8()?;
        Ok(Self { protocol_version })
    }
}
implement_packet!(OpenConnectionRequest1, PacketId::OPEN_CONNECTION_REQUEST_1);

#[derive(Debug, Clone)]
pub struct OpenConnectionReply1 {
    pub server_guid: u64,
    pub security: bool,
    pub mtu_size: u16,
}

impl OpenConnectionReply1 {
    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_magic()?;
        writer.write_u64_be(self.server_guid)?;
        writer.write_bool(self.security)?;
        writer.write_u16_be(self.mtu_size)?;
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        if !reader.read_magic()? {
            return Err(binary::BinaryError::InvalidData(
                "Invalid magic sequence in OpenConnectionReply1".into(),
            ));
        }
        let server_guid = reader.read_u64_be()?;
        let security = reader.read_bool()?;
        let mtu_size = reader.read_u16_be()?;
        if mtu_size < 500 || mtu_size > crate::consts::MAX_MTU {
            return Err(binary::BinaryError::InvalidData(format!("Invalid MTU size: {}", mtu_size)));
        }
        Ok(Self {
            server_guid,
            security,
            mtu_size,
        })
    }
}
implement_packet!(OpenConnectionReply1, PacketId::OPEN_CONNECTION_REPLY_1);

#[derive(Debug, Clone)]
pub struct OpenConnectionRequest2 {
    pub server_address_ignored: SocketAddr,
    pub mtu_size: u16,
    pub client_guid: u64,
}
impl OpenConnectionRequest2 {
    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_magic()?;
        writer.write_address(&self.server_address_ignored)?;
        writer.write_u16_be(self.mtu_size)?;
        writer.write_u64_be(self.client_guid)?;
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        if !reader.read_magic()? {
            return Err(BinaryError::InvalidData(
                "Invalid magic sequence in OpenConnectionRequest2".into(),
            ));
        }
        let server_address_ignored = reader.read_address()?;
        let mtu_size = reader.read_u16_be()?;
        let client_guid = reader.read_u64_be()?;
        Ok(Self {
            server_address_ignored,
            mtu_size,
            client_guid,
        })
    }
}
implement_packet!(OpenConnectionRequest2, PacketId::OPEN_CONNECTION_REQUEST_2);

#[derive(Debug, Clone)]
pub struct OpenConnectionReply2 {
    pub server_guid: u64,
    pub client_address: SocketAddr,
    pub mtu_size: u16,
    pub security: bool,
}

impl OpenConnectionReply2 {
    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_magic()?;
        writer.write_u64_be(self.server_guid)?;
        writer.write_address(&self.client_address)?;
        writer.write_u16_be(self.mtu_size)?;
        writer.write_bool(self.security)?;
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        if !reader.read_magic()? {
            return Err(BinaryError::InvalidData(
                "Invalid magic sequence in OpenConnectionReply2".into(),
            ));
        }
        let server_guid = reader.read_u64_be()?;
        let client_address = reader.read_address()?;
        let mtu_size = reader.read_u16_be()?;
        let security = reader.read_bool()?;
        Ok(Self {
            server_guid,
            client_address,
            mtu_size,
            security,
        })
    }
}
implement_packet!(OpenConnectionReply2, PacketId::OPEN_CONNECTION_REPLY_2);


#[derive(Debug, Clone)]
pub struct ConnectionRequest {
    pub client_guid: u64,
    pub request_timestamp: i64,
    pub use_security: bool,
}

impl ConnectionRequest {
    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_u64_be(self.client_guid)?;
        writer.write_i64_be(self.request_timestamp)?;
        writer.write_bool(self.use_security)?;
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        let client_guid = reader.read_u64_be()?;
        let request_timestamp = reader.read_i64_be()?;
        let use_security = reader.read_bool()?;
        Ok(Self {
            client_guid,
            request_timestamp,
            use_security,
        })
    }
}
implement_packet!(ConnectionRequest, PacketId::CONNECTION_REQUEST);


#[derive(Debug, Clone)]
pub struct ConnectionRequestAccepted {
    pub client_address: SocketAddr,
    pub system_index: u16,
    pub request_timestamp: i64,
    pub accepted_timestamp: i64,
}

impl ConnectionRequestAccepted {
    const PADDING_ADDRESS: SocketAddr = SocketAddr::V4(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::new(255, 255, 255, 255),
        19132,
    ));

    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_address(&self.client_address)?;
        writer.write_u16_be(self.system_index)?;
        for _ in 0..10 {
            writer.write_address(&Self::PADDING_ADDRESS)?;
        }
        writer.write_i64_be(self.request_timestamp)?;
        writer.write_i64_be(self.accepted_timestamp)?;
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        let client_address = reader.read_address()?;
        let system_index = reader.read_u16_be()?;
        // Apparently, after researching, normal RakNet uses 10, but MC-BE uses 20.
        for _ in 0..20 {
            let _ = reader.read_address()?;
        }
        let request_timestamp = reader.read_i64_be()?;
        let accepted_timestamp = reader.read_i64_be()?;
        Ok(Self {
            client_address,
            system_index,
            request_timestamp,
            accepted_timestamp,
        })
    }
}
implement_packet!(ConnectionRequestAccepted, PacketId::CONNECTION_REQUEST_ACCEPTED);


#[derive(Debug, Clone)]
pub struct NewIncomingConnection {
    pub server_address: SocketAddr,
    pub request_timestamp: i64,
    pub accepted_timestamp: i64,
}

impl NewIncomingConnection {
    const PADDING_ADDRESS: SocketAddr = SocketAddr::V4(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::new(0, 0, 0, 0),
        0,
    ));

    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_address(&self.server_address)?;
        // Apparently, after researching, normal RakNet uses 10, but MC-BE uses 20.
        for _ in 0..20 {
            writer.write_address(&Self::PADDING_ADDRESS)?;
        }
        writer.write_i64_be(self.request_timestamp)?;
        writer.write_i64_be(self.accepted_timestamp)?;
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        let server_address = reader.read_address()?;
        // Apparently, after researching, normal RakNet uses 10, but MC-BE uses 20.
        for _ in 0..20 {
            let _ = reader.read_address()?;
        }
        let request_timestamp = reader.read_i64_be()?;
        let accepted_timestamp = reader.read_i64_be()?;
        Ok(Self {
            server_address,
            request_timestamp,
            accepted_timestamp,
        })
    }
}
implement_packet!(NewIncomingConnection, PacketId::NEW_INCOMING_CONNECTION);

#[derive(Debug, Clone)]
pub struct IncompatibleProtocolVersion {
    pub protocol_version: u8,
    pub server_guid: u64,
}

impl IncompatibleProtocolVersion {
    pub fn encode_payload(&self, writer: &mut impl BinaryWriter) -> BinaryResult<()> {
        writer.write_u8(self.protocol_version)?;
        writer.write_magic()?;
        writer.write_u64_be(self.server_guid)?;
        Ok(())
    }

    pub fn decode_payload(reader: &mut impl BinaryReader) -> BinaryResult<Self> {
        let protocol_version = reader.read_u8()?;
        if !reader.read_magic()? {
            return Err(BinaryError::InvalidData(
                "Invalid magic sequence in IncompatibleProtocolVersion".into(),
            ));
        }
        let server_guid = reader.read_u64_be()?;
        Ok(Self {
            protocol_version,
            server_guid,
        })
    }
}
implement_packet!(IncompatibleProtocolVersion, PacketId::INCOMPATIBLE_PROTOCOL_VERSION);