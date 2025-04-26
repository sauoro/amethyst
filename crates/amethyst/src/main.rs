use log::{debug, error, info, logger, set_logger, set_max_level, Level, SetLoggerError};
use tokio::time::Instant;
use amethyst_log::AmethystLogger;

pub mod config;

#[tokio::main]
async fn main() -> Result<(), SetLoggerError> {
    AmethystLogger::init(Level::Info).unwrap();
    
    let start_time = Instant::now();

    let config = match config::handle() {
        Ok(config) => config,
        Err(_e) => {
            error!("Failed to load configuration.");
            std::process::exit(1);
        }
    };

    info!("Server started");
    debug!("This won't be logged because level is Info");
    error!("An error occurred");
    logger().flush();

    let server_name = &config.server.name;
    let elapsed_duration = start_time.elapsed();

    info!(
        "{}'s load done in {:.2}s",
        server_name,
        elapsed_duration.as_secs_f64()
    );
    
    Ok(())
}