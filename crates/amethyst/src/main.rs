use tokio::time::Instant;

pub mod config;

#[tokio::main]
async fn main() {
    let start_time = Instant::now();

    let config = match config::handle() {
        Ok(config) => config,
        Err(_e) => {
            println!("Failed to load or create configuration.");
            std::process::exit(1);
        }
    };

    let server_name = &config.server.name;
    let elapsed_duration = start_time.elapsed();

    println!(
        "{}'s loading done in {:.2}s",
        server_name,
        elapsed_duration.as_secs_f64()
    );
}
