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
    /// Whether the applications acts as a client or server
    #[clap(short, long)]
    pub server: bool,
    /// Server port
    #[clap(short, long)]
    pub port: Option<u16>,
    /// Server address
    #[clap(long)]
    pub server_address: Option<Multiaddr>,
    /// Remote server peer id
    #[clap(long)]
    pub server_peer_id: Option<PeerId>,
    /// Key seed to generate keypair
    #[clap(long)]
    pub key_seed: Option<u8>,
    /// Whether to use IPV6 or IPV4
    #[clap(long)]
    pub use_ipv6: Option<bool>,
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
