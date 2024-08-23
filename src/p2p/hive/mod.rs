use clap::Parser;
use libp2p::{
    multiaddr::Multiaddr,
    PeerId,
};
use std::error::Error;

pub mod behavior;
pub mod client;
pub mod server;

#[derive(Parser)]
pub struct HiveParameters {
    #[clap(short, long)]
    pub server: bool,
    #[clap(short, long)]
    pub listen_port: Option<u16>,
    #[clap(long)]
    pub server_address: Option<Multiaddr>,
    #[clap(long)]
    pub server_peer_id: Option<PeerId>,
}

/// Main function
/// 
/// 
pub async fn main(parameters: HiveParameters) -> Result<(), Box<dyn Error>> {
    if parameters.server {
        server::main(parameters).await?;
    } else {
        client::main(parameters).await?;
    }
    
    Ok(())
}
