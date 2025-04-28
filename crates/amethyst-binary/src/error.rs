use thiserror::Error;

#[derive(Error, Debug)]
pub enum BinaryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Varint too large for target type")]
    VarintTooLarge,
    #[error("Buffer ended unexpectedly")]
    UnexpectedEOF,
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub type Result<T> = std::result::Result<T, BinaryError>;