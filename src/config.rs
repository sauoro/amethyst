use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use log::{Log, Logger};

const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Failed to get current directory: {0}")]
    CurrentDir(io::Error),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub bind_address: String,
    pub max_players: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                name: "Amethyst".to_string(),
                bind_address: "0.0.0.0:19132".to_string(),
                max_players: 50,
            },
        }
    }
}

pub fn initialize() -> Result<Config> {
    let logger: Logger = Logger::nameless();

    let config_path = PathBuf::from(CONFIG_FILE_NAME);

    if config_path.exists() {
        let config_content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    } else {
        logger.info(format!(
            "Configuration file '{}' not found. Creating default configuration.",
            CONFIG_FILE_NAME
        ).as_str());
        let config = Config::default();
        save(&config, &config_path)?;
        logger.info(format!("Default configuration saved to '{}'.", CONFIG_FILE_NAME).as_str());
        Ok(config)
    }
}

fn save(config: &Config, path: &Path) -> Result<()> {
    let config_content = toml::to_string_pretty(config)?;
    let mut file = fs::File::create(path)?;
    file.write_all(config_content.as_bytes())?;
    Ok(())
}