
// src/main.rs
use raknet::RakNetServer;
use std::net::SocketAddr;
use tracing::Level;

// Make the binary module public at the crate root
pub mod utils;
// Make the raknet module public at the crate root
pub mod raknet;
// Expose the BinaryReader and BinaryWritter traits at the crate root
pub use utils::binary::{BinaryReader, BinaryWritter};

// Define a basic error type for the main function, could use anyhow later
#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("RakNet server error: {0}")]
    RakNet(#[from] raknet::error::RakNetError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// Type alias for the main result type
type Result<T> = std::result::Result<T, AppError>;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup tracing subscriber for logging
    // Use INFO level for now, can be configured later
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Define the server address and port
    // Using 0.0.0.0 allows binding to all available network interfaces
    let bind_addr: SocketAddr = "0.0.0.0:19132".parse().expect("Invalid Socket Address");

    // Create and run the RakNet server
    tracing::info!("Starting Amethyst RakNet server on {}...", bind_addr);
    let server = RakNetServer::bind(bind_addr).await?;

    // The server's run loop will handle connections and packet processing
    // We can add logic here to handle application-level events from the server if needed
    // For now, just run the server until it stops or errors out.
    server.run().await?;

    tracing::info!("Amethyst RakNet server stopped.");
    Ok(())
}
