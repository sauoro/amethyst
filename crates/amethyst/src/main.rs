use std::error::Error;
use log::{debug, error, info, logger, trace, warn, Level};
use rak_rs::connection::Connection;
use rak_rs::Listener;
use rak_rs::mcpe::motd::Gamemode;
use tokio::time::{sleep, Instant, Duration};
use amethyst_log::AmethystLogger;

pub mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = AmethystLogger::init(Level::Trace, 1024) {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }
    
    let start_time = Instant::now();

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

    let elapsed_duration = start_time.elapsed();
    info!("Server load done in {:.2}s",elapsed_duration.as_secs_f64());
    logger().flush();

    let address = &config.network.address;
    let mut server = match Listener::bind(address.as_str()).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to bind listener to {}: {}", address, e);
            logger().flush();
            return Err(e.into());
        }
    };
    
    server.motd.name = config.server.name;
    server.motd.sub_name = "Amethyst".to_owned();
    server.motd.gamemode = Gamemode::Survival;
    
    server.start().await.unwrap();

    info!("Listening on {}", &config.network.address);
    logger().flush();
    
    loop {
        tokio::select! {
            conn_result = server.accept() => {
                match conn_result {
                    Ok(conn) => {
                        info!("Accepted new connection from: {}", &conn.address);
                        logger().flush();
                        tokio::spawn(handle_connection(conn));
                    }
                    Err(e) => {
                        error!("Error accepting connection: {}", e);
                        logger().flush();
                        sleep(Duration::from_millis(1000)).await;
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl+C received, shutting down.");
                logger().flush();
                break;
            }
        }
    }
    
    info!("Server shutting down...");
    logger().flush();
    
    sleep(Duration::from_millis(1000)).await;
    Ok(())
}

async fn handle_connection(mut conn: Connection) {
    debug!("[{}] Starting connection handler task.", &conn.address);
    logger().flush();
    
    loop {
        tokio::select! {
            result = conn.recv() => {
                match result {
                    Ok(buffer) => {
                        trace!(
                            "[{}] Received packet: Size={}, Preview={:02X?}",
                            &conn.address,
                            buffer.len(),
                            buffer.get(..std::cmp::min(buffer.len(), 16)).unwrap_or(&[])
                        );
                        logger().flush();
                        
                        if let Some(packet_id) = buffer.get(0) {
                            match *packet_id {
                                0xfe => {
                                    info!("[{}] Received Game Packet Batch (0xFE), Size: {}", &conn.address, buffer.len());
                                    logger().flush();
                                }
                                unknown_id => {
                                    warn!(
                                        "[{}] Received unknown application packet ID: {:#02X}, Size: {}",
                                        &conn.address, unknown_id, buffer.len()
                                    );
                                    logger().flush();
                                }
                            }
                        } else {
                            debug!("[{}] Received empty buffer.", &conn.address);
                            logger().flush();
                        }

                    }
                    Err(e) => {
                        if conn.is_closed().await {
                            debug!("[{}] recv() failed because connection is closed.", &conn.address);
                            logger().flush();
                        } else {
                            error!("[{}] Error receiving packet: {}", &conn.address, e);
                            logger().flush();
                        }
                        break;
                    }
                }
            }
        }
    }
}