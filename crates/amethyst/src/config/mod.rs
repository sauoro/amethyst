use error::ConfigError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub mod error;

const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub network: NetworkConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub fn validate(&self) -> Result<(), ConfigError> {
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

pub fn handle() -> Result<Config, ConfigError> {
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

fn save(config: &Config, path: &Path) -> Result<(), ConfigError> {
    let config_content = toml::to_string_pretty(config)?;
    let mut file = fs::File::create(path)?;
    file.write_all(config_content.as_bytes())?;
    Ok(())
}