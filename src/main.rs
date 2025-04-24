pub mod config;
use log::{Log, Logger};

#[tokio::main]
async fn main() {
    let logger = Logger::nameless();

    let config = match config::initialize() {
        Ok(cfg) => {
            logger.info("Configuration loaded successfully.");
            cfg
        }
        Err(e) => {
            logger.error(&format!("Failed to load or create configuration: {}", e));
            eprintln!("Critical error: Could not load configuration. Exiting.");
            std::process::exit(1);
        }
    };

    logger.info(&format!("Server name: {}", config.server.name));
    logger.info(&format!(
        "Binding to address: {}",
        config.server.bind_address
    ));
    logger.info(&format!("Max players: {}", config.server.max_players));

    logger.info("Amethyst server starting...");

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    logger.info("Amethyst server shutting down.");
}