//! Distributed computing server using Actix-web and multicast discovery
//! 
//! 
use clap::{Parser, Subcommand};
use tokio;
use std::env;

pub mod config;
pub mod multicast;
pub mod rest_server;

use multicast::udp_socket::{
    client,
    server
};

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    // Start the server
    Server,
    // Start the client
    Client,
    RestServer,
}

#[cfg(debug_assertions)]
fn set_debug() {
    env::set_var("DEBUG", "TRUE");
}

/// Main function
/// 
/// 
#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    set_debug();
    
    let cli = Cli::parse();
    
    match cli.command {
        Command::Server => {
            if let Err(e) = server::start_server().await {
                eprintln!("Error starting server: {}", e);
            } else {
                println!("Server started successfully!");
            }
        }
        Command::Client => {
            if let Err(e) = client::start_client().await {
                eprintln!("Error starting client: {}", e);
            } else {
                println!("Client started successfully!");
            }
        }
        Command::RestServer => {
            if let Err(e) = rest_server::start_rest_server().await {
                eprintln!("Error starting rest server: {}", e);
            } else {
                println!("Rest server started successfully!");
            }
        }
    };

    Ok(())
}

