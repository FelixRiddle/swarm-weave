/// Distributed computing server using Actix-web and multicast discovery
use clap::{Parser, Subcommand};
use tokio;

pub mod client;
pub mod rest_server;
pub mod server;
pub mod config;

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

/// Main function
/// 
/// 
#[tokio::main]
pub async fn main() -> std::io::Result<()> {
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

