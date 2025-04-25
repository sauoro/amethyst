pub const RAKNET_PROTOCOL_VERSION: u8 = 11;
pub const RAKNET_MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];
pub const DEFAULT_MTU: u16 = 1400;
pub const MAX_MTU: u16 = 1492;

pub const DEFAULT_CONNECT_TIMEOUT_MS: u64 = 5000;
pub const DEFAULT_SESSION_TIMEOUT_MS: u64 = 30000;
pub const DEFAULT_KEEP_ALIVE_INTERVAL_MS: u64 = 8000;
