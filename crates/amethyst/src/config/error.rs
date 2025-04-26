use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("Failed to parse TOML from file '{path}': {source}")]
    TomlDeserialization {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("Failed to serialize configuration to TOML for file '{path}': {source}")]
    TomlSerialization {
        path: String,
        #[source]
        source: toml::ser::Error,
    },
    #[error("Configuration validation failed: {reason}")]
    Validation { reason: String },
    #[error("Failed to determine current directory: {0}")]
    CurrentDirectory(#[from] io::Error),
    #[error("Configuration file not found at '{path}'")]
    FileNotFound { path: String },
    #[error("Permission denied while accessing '{resource}': {details}")]
    PermissionDenied {
        resource: String,
        details: String,
    },
    #[error("Invalid configuration value for '{field}': {details}")]
    InvalidValue {
        field: String,
        details: String,
    },
    #[error("Unexpected file format for '{path}': expected {expected}, found {found}")]
    UnexpectedFormat {
        path: String,
        expected: String,
        found: String,
    },
}

pub type Result<T> = std::result::Result<T, ConfigError>;
