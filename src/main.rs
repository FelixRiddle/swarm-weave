//! Distributed computing server using Actix-web and multicast discovery
//! 
//! 
use clap::{Parser, Subcommand};
use config::env::set_debug;
use server::StartServerOptions;
use std::error::Error;
use tokio;

pub mod client;
pub mod config;
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
pub async fn main() -> Result<(), Box<dyn Error>> {
    set_debug();
    
    let cli = Cli::parse();
    
    match cli.command {
        Command::Server => {
            let options = StartServerOptions::default_controlled()?;
            if let Err(e) = server::start_server(options).await {
                eprintln!("Error starting server: {}", e);
            } else {
                println!("Server started successfully!");
            }
        }
        Command::Client => {
            // Run egui app in a separate thread
            std::thread::spawn(client::gui::main);
        }
    };
    
    Ok(())
}

