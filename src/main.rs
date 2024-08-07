//! Distributed computing server using Actix-web and multicast discovery
//! 
//! 
use clap::{Parser, Subcommand};
use config::env::set_debug;
use tokio;

pub mod client;
pub mod config;
pub mod multicast;
pub mod server;

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
            if let Err(e) = client::gui::main().await {
                eprintln!("Error starting client: {}", e);
            } else {
                println!("Client started successfully!");
            }
        }
    };
    
    Ok(())
}

