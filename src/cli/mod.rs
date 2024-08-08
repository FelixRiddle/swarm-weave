use std::error::Error;

use clap::{Parser, Subcommand};
use crate::server::{
    self,
    StartServerOptions,
};
use crate::client;

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

/// Main
/// 
/// 
pub async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Command::Server => {
            let options = StartServerOptions::default_controlled()?;
            if let Err(e) = server::start_server(options).await {
                eprintln!("Error starting server: {}", e);
            } else {
                println!("Server closed");
            }
        }
        Command::Client => {
            // Run egui app in a separate thread
            std::thread::spawn(client::gui::main);
        }
    };
    
    Ok(())
}
