use libp2p_identity::Keypair;
use libp2p::swarm::NetworkBehaviour;
use libp2p::{
    autonat,
    gossipsub,
    identify,
    identity,
    mdns,
    ping,
    relay,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::io;

/// Generate ed25519
/// 
/// 
pub fn generate_ed25519(secret_key_seed: u8) -> Result<identity::Keypair, Box<dyn Error>> {
    let mut bytes = [0u8; 32];
    bytes[0] = secret_key_seed;
    
    Ok(identity::Keypair::ed25519_from_bytes(bytes)?)
}

/// We create a custom network behaviour that combines Gossipsub and Mdns.
/// 
/// This macro creates 'MyBehaviorEvent'
#[derive(NetworkBehaviour)]
pub struct MyBehavior {
    pub auto_nat: autonat::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
    pub relay: relay::Behaviour,
}

impl MyBehavior {
    /// Create new behavior
    /// 
    /// 
    pub fn new(key: &Keypair) -> Result<Self, Box<dyn Error>> {
        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };
        
        // Set a custom gossipsub configuration
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
            .build()
            .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.
        
        // build a gossipsub network behaviour
        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            gossipsub_config,
        )?;
        
        let mdns =
            mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
        
        Ok(MyBehavior {
            auto_nat: autonat::Behaviour::new(
                key.public().to_peer_id(),
                autonat::Config {
                    only_global_ips: false,
                    ..Default::default()
                }
            ),
            gossipsub,
            mdns,
            identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/0.1.0".into(),
                key.public()
            )),
            relay: relay::Behaviour::new(key.public().to_peer_id(), Default::default()),
            ping: ping::Behaviour::new(ping::Config::new()),
        })
    }
}
