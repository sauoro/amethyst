use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDeserialization(#[from] toml::de::Error),
    #[error("TOML serialization error: {0}")]
    TomlSerialization(#[from] toml::ser::Error),
    #[error("Configuration validation failed: {0}")]
    Validation(String),
    #[error("Failed to get current directory: {0}")]
    CurrentDirectory(io::Error),
}

pub type Result<T> = std::result::Result<T, ConfigError>;
