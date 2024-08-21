//! Distributed computing server using Actix-web and multicast discovery
//! 
//! 
use config::env::set_debug;
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

/// Main function
/// 
/// 
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    // Parse environment variables
    dotenv().ok();
    
    set_debug();
    
    cli::main().await?;
    
    Ok(())
}

