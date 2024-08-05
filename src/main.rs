/// Distributed computing server using Actix-web and multicast discovery
use tokio;

pub mod client;
pub mod server;

/// Main function
/// 
/// 
#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    // Start client server and print a message on success and on error
    if let Err(e) = client::start_client().await {
        eprintln!("Error starting client: {}", e);
    } else {
        println!("Client started successfully!");
    }
    
    Ok(())
}

