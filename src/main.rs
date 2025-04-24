use tokio::time::Instant;
use log::{Log, Logger};

pub mod config;

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    let system_logger = Logger::nameless();

    let config = match config::handle() {
        Ok(config) => config,
        Err(_e) => {
            system_logger.error("Failed to load or create configuration.");
            std::process::exit(1);
        }
    };

    let server_name = &config.server.name;

    let elapsed_duration = start_time.elapsed();
    let elapsed_secs_f64 = elapsed_duration.as_secs_f64();

    system_logger.info(format!(
        "Finished loading server '{}' in {:.2}s",
        server_name,
        elapsed_secs_f64
    ).as_str());
}
