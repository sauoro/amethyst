use log::{error, info, set_logger, set_max_level, SetLoggerError};
use tokio::time::Instant;

pub mod config;

#[tokio::main]
async fn main() -> Result<(), SetLoggerError> {
    let _ = set_logger(&lib::AMETHYST_LOGGER);
    set_max_level(log::LevelFilter::Info);
    
    let start_time = Instant::now();

    let config = match config::handle() {
        Ok(config) => config,
        Err(_e) => {
            error!("Failed to load configuration.");
            std::process::exit(1);
        }
    };

    let server_name = &config.server.name;
    let elapsed_duration = start_time.elapsed();

    info!(
        "{}'s load done in {:.2}s",
        server_name,
        elapsed_duration.as_secs_f64()
    );
    
    Ok(())
}