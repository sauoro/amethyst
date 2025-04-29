use std::error::Error;
use std::sync::Arc;
use log::{error, info, logger, Level};
use tokio::time::{Instant, Duration};
use amethyst_log::AmethystLogger;
use crate::config::Config;
use tokio::signal;
use rakethyst::listener::RakNetListener;

pub mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    rakethyst::utils::init_time();
    
    if let Err(e) = AmethystLogger::init(Level::Trace, 1024) {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }

    let start_time = Instant::now();

    let config: Arc<Config> = match config::handle() {
        Ok(config) => {
            info!("Configuration loaded successfully.");
            logger().flush();
            Arc::new(config)
        },
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            tokio::time::sleep(Duration::from_secs(5)).await;
            return Err(e.into());
        }
    };
    let listener = match RakNetListener::bind(&config.network.address, config.server.name.clone()).await {
        Ok(listener) => listener,
        Err(e) => {
            error!(
                "Failed to bind RakNet listener to {}: {}",
                config.network.address, e
            );
            return Err(e);
        }
    };

    let elapsed_duration = start_time.elapsed();
    info!(
        "Server startup complete in {:.3}s. Listening on {}",
        elapsed_duration.as_secs_f64(),
        config.network.address
    );
    logger().flush();

    tokio::select! {
        res = listener.run() => {
            if let Err(e) = res {
                error!("RakNet listener exited with error: {}", e);
            } else {
                info!("RakNet listener stopped gracefully.");
            }
        }
        _ = signal::ctrl_c() => {
            info!("Ctrl+C received, initiating shutdown...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    info!("Shutting down server.");
    logger().flush();
    Ok(())
}