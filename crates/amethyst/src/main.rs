use log::{error, info, logger, Level, SetLoggerError};
use tokio::time::{sleep, Instant, Duration};
use amethyst_log::AmethystLogger;

pub mod config;

#[tokio::main]
async fn main() -> Result<(), SetLoggerError> {
    if let Err(e) = AmethystLogger::init(Level::Info, 1024) {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }
    
    let start_time = Instant::now();

    info!("Loading configuration");
    let config = match config::handle() {
        Ok(config) => {
            info!("Configuration loaded successfully.");
            logger().flush();
            config
        },
        Err(_e) => {
            error!("Failed to load configuration.");
            logger().flush();
            tokio::time::sleep(Duration::from_secs(5)).await;
            std::process::exit(1);
        }
    };

    info!("Loading extras...");
    
    sleep(Duration::from_secs(3)).await;

    let server_name = &config.server.name;
    let elapsed_duration = start_time.elapsed();

    info!(
        "{}'s load done in {:.2}s",
        server_name,
        elapsed_duration.as_secs_f64()
    );

    sleep(Duration::from_secs(1)).await;

    info!("Server shutting down gracefully...");

    logger().flush();
    
    Ok(())
}