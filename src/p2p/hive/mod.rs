use clap::Parser;
use libp2p::{
    multiaddr::Multiaddr,
    PeerId,
};
use rand::Rng;
use std::error::Error;

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
    /// Whether it's relay or not
    #[clap(long)]
    pub relay: bool,
}

impl Default for HiveParameters {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let key_seed = rng.gen::<u8>();

        HiveParameters {
            server: true,
            port: None,
            server_address: None,
            server_peer_id: None,
            key_seed: Some(key_seed),
            use_ipv6: None,
            relay: false,
        }
    }
}

impl HiveParameters {
    pub fn generate_peer_id(&self) -> Result<PeerId, Box<dyn Error>> {
        let key = libp2p::identity::Keypair::generate_ed25519();
        Ok(key.public().to_peer_id())
    }
    
    pub fn get_port(&self) -> u16 {
        let port = match self.port {
            Some(port) => port,
            None => 0,
        };
        
        port
    }
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
