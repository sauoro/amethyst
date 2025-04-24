use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    
    #[error("Configuration validation failed: {0}")]
    Validation(String),
    
    #[error("Failed to get current directory: {0}")]
    CurrentDir(io::Error),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub network: NetworkConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub max_players: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:19132".to_string(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "Amethyst".to_string(),
            max_players: 50,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if SocketAddr::from_str(&self.network.address).is_err() {
            return Err(ConfigError::Validation(format!(
                "Invalid network address format: '{}'. Expected format like 'IP:PORT'.",
                self.network.address
            )));
        }

        if self.server.name.trim().is_empty() {
            return Err(ConfigError::Validation(
                "Server name cannot be empty.".to_string(),
            ));
        }

        if self.server.max_players == 0 {
            return Err(ConfigError::Validation(
                "Maximum players must be greater than 0.".to_string(),
            ));
        }

        Ok(())
    }
}

pub fn handle() -> Result<Config> {
    let config_path = PathBuf::from(CONFIG_FILE_NAME);
    if config_path.exists() {
        let config_content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&config_content)?;
        config.validate()?;
        Ok(config)
    } else {
        let config = Config::default();
        save(&config, &config_path)?;
        Ok(config)
    }
}

fn save(config: &Config, path: &Path) -> Result<()> {
    let config_content = toml::to_string_pretty(config)?;
    let mut file = fs::File::create(path)?;
    file.write_all(config_content.as_bytes())?;
    Ok(())
}