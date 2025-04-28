use amethyst_binary::error::BinaryError;
use amethyst_binary::error::BinaryError::InvalidData;
use amethyst_binary::io::{BinaryReader, BinaryWriter};
use amethyst_binary::traits::{Readable, Writable};

pub const MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

pub const CONNECTED_PING: u8 = 0x00;
pub const UNCONNECTED_PING: u8 = 0x01;
pub const UNCONNECTED_PING_OPEN_CONNECTIONS: u8 = 0x02;
pub const CONNECTED_PONG: u8 = 0x03;
pub const UNCONNECTED_PONG: u8 = 0x1c;

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
            Err(InvalidData("Expected magic bytes.".to_string()))?
        }
        let motd = reader.read_string_u16()?;
        Ok(Self {
            time,
            server_guid,
            motd,
        })
    }
}
