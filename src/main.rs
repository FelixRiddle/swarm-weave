//! Distributed computing server using Actix-web and multicast discovery
//! 
//! 
use dotenv::dotenv;
use std::error::Error;
use tokio;

pub mod cli;
pub mod client;
pub mod config;
pub mod database;
pub mod model;
pub mod p2p;
pub mod security;
pub mod server;
pub mod server_node;
pub mod test;

#[cfg(debug_assertions)]
use config::env::set_debug;

/// Main function
/// 
/// 
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    // Parse environment variables
    dotenv().ok();
    
    #[cfg(debug_assertions)]
    set_debug();
    
    cli::main().await?;
    
    Ok(())
}

