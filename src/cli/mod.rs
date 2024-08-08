use std::error::Error;
use clap::{Parser, Subcommand};

use crate::client;
use crate::database;
use crate::server::{
    self,
    StartServerOptions,
};

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the server
    Server {
        #[clap(short, long)]
        port: Option<u16>,
    },
    /// Start the client
    Client,
    /// Print
    Print {
        /// Show mysql connection string
        #[clap(short, long)]
        mysql_connection_string: bool,
    },
}

/// Main
/// 
/// 
pub async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Command::Server { port } => {
            // Create options
            let mut options = StartServerOptions::default_controlled()?;
            
            options.port = port.unwrap_or(options.port);
            
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
        Command::Print {
            mysql_connection_string,
        } => {
            if mysql_connection_string {
                // Don't change!, there's a script that relies on this output
                println!("{}", database::mysql_connection_string());
            }
        }
    };
    
    Ok(())
}
