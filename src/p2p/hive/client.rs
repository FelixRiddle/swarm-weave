use futures::StreamExt;
use libp2p::{
    noise,
    tcp,
    yamux,
};
use libp2p::core::Multiaddr;
use libp2p::core::multiaddr::Protocol;
use libp2p::swarm::SwarmEvent;
use std::error::Error;
use std::net::Ipv4Addr;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

use super::HiveParameters;
use crate::p2p::node::behavior::MyBehavior;

/// Hive client
/// 
/// 
pub async fn main(parameters: HiveParameters) -> Result<(), Box<dyn Error>> {
    
    // let address = match parameters.server_address {
    //     Some(address) => address,
    //     None => return Err("Server address is required".into()),
    // };
    
    // let server_peer_id = match parameters.server_peer_id {
    //     Some(peer_id) => peer_id,
    //     None => return Err("Server peer_id is required".into()),
    // };
    
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
    
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default
        )?
        .with_behaviour(|key| Ok(MyBehavior::new(key).unwrap()))?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();
    
    swarm.listen_on(
        Multiaddr::empty()
            .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(parameters.port.unwrap_or(0)))
    )?;
    
    // // Auto nat seems to not exist, dropping it sadly
    // swarm.behaviour_mut()
    //     .auto_nat
    //     .add_server(server_peer_id, Some(address));
    
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address:?}");
            }
            SwarmEvent::Behaviour(event) => {
                println!("{event:?}");
            }
            e => {
                println!("{e:?}");
            }
        }
    }
}
