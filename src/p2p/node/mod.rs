use futures::executor::block_on;
use futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{
    gossipsub,
    identify,
    identity,
    mdns,
    multiaddr::Protocol,
    noise,
    tcp,
    yamux,
    Multiaddr,
    SwarmBuilder
};
use std::error::Error;
use std::net::{
    Ipv4Addr,
    Ipv6Addr,
};
use std::time::Duration;
use tokio::{io, io::AsyncBufReadExt, select};
use tracing_subscriber::EnvFilter;

use crate::p2p::hive::HiveParameters;

pub mod behavior;

use behavior::{generate_ed25519, MyBehavior, MyBehaviorEvent};

/// Use this computer to join the swarm network
/// 
/// 
pub struct Node {
    pub parameters: HiveParameters,
    pub local_key: identity::Keypair,
    pub swarm: libp2p::Swarm<MyBehavior>,
    pub topic: gossipsub::IdentTopic,
}

impl Node {
    pub async fn new(parameters: HiveParameters) -> Result<Self, Box<dyn Error>> {
        // Fetch key seed
        let key_seed = match parameters.key_seed {
            Some(seed) => seed,
            None => return Err("Key seed is required".into()),
        };
        
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .try_init();
        
        let local_key: identity::Keypair = generate_ed25519(key_seed)?;
        
        // Create swarm
        let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|key| {
                Ok(MyBehavior::new(key).unwrap())
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();
        
        // Create a Gossipsub topic
        let topic = gossipsub::IdentTopic::new("chat-net");
        // subscribes to our topic
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        
        Ok(Node {
            parameters,
            local_key,
            swarm,
            topic,
        })
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let port = match self.parameters.port {
            Some(port) => port,
            None => 0,
        };
        
        // Relay
        // Listen on all interfaces
        let listen_addr_tcp = Multiaddr::empty()
            .with(match self.parameters.use_ipv6 {
                Some(true) => Protocol::Ip6(Ipv6Addr::UNSPECIFIED),
                Some(false) => Protocol::Ip4(Ipv4Addr::UNSPECIFIED),
                None => Protocol::Ip4(Ipv4Addr::UNSPECIFIED),
            })
            .with(Protocol::Tcp(port));
        self.swarm.listen_on(listen_addr_tcp)?;
        
        let listen_addr_quic = Multiaddr::empty()
            .with(match self.parameters.use_ipv6 {
                Some(true) => Protocol::Ip6(Ipv6Addr::UNSPECIFIED),
                Some(false) => Protocol::Ip4(Ipv4Addr::UNSPECIFIED),
                None => Protocol::Ip4(Ipv4Addr::UNSPECIFIED)
            })
            .with(Protocol::Udp(port))
            .with(Protocol::QuicV1);
        self.swarm.listen_on(listen_addr_quic)?;
        
        // Read lines from stdin
        let mut stdin = io::BufReader::new(io::stdin()).lines();
        
        // Listen on all interfaces and whatever port the OS assigns
        self.swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        
        // Autonat server
        self.swarm.listen_on(
            Multiaddr::empty()
                .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
                .with(Protocol::Tcp(0)),
        )?;
        
        println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");
        
        // Kick it off
        block_on(async {
            loop {
                select! {
                    Ok(Some(line)) = stdin.next_line() => {
                        if let Err(e) = self.swarm
                            .behaviour_mut().gossipsub
                            .publish(self.topic.clone(), line.as_bytes()) {
                            println!("Publish error: {e:?}");
                        }
                    }
                    event = self.swarm.select_next_some() => match event {
                        SwarmEvent::Behaviour(event) => {
                            println!("{event:?}");
                            
                            match event {
                                // MyBehavior event is an enum created with select!
                                // MDNS
                                MyBehaviorEvent::Mdns(event) => {
                                    match event {
                                        mdns::Event::Discovered(list) => {
                                            for (peer_id, multiaddr) in list {
                                                println!("mDNS discovered a new peer: {peer_id}");
                                                
                                                // I need to fetch node name and information
                                                // Extract IP address from multiaddr
                                                let components: Vec<_> = multiaddr.iter().collect();
                                                match components[0] {
                                                    Protocol::Ip4(ipv4_addr) => println!("IP address: {}", ipv4_addr),
                                                    Protocol::Ip6(ipv6_addr) => println!("IP address: {}", ipv6_addr),
                                                    _ => println!("Unsupported protocol"),
                                                }
                                                
                                                self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                            }
                                        }
                                        mdns::Event::Expired(list) => {
                                            for (peer_id, _multiaddr) in list {
                                                println!("mDNS discover peer has expired: {peer_id}");
                                                self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                                            }
                                        }
                                    }
                                }
                                // Gossipsub
                                MyBehaviorEvent::Gossipsub(gossipsub::Event::Message {
                                    propagation_source: peer_id,
                                    message_id: id,
                                    message,
                                }) => {
                                    println!(
                                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                                        String::from_utf8_lossy(&message.data),
                                    );
                                }
                                MyBehaviorEvent::Identify(identify::Event::Received {
                                    info: identify::Info { observed_addr, .. },
                                    ..
                                }) =>{
                                    self.swarm.add_external_address(observed_addr.clone());
                                }
                                _ => {}
                            }
                        }
                        SwarmEvent::NewListenAddr { listener_id, address, } => {
                            println!("Local node {} is listening on {address}", listener_id.to_string());
                        }
                        _ => {}
                    }
                }
            }
        });
        
        Ok(())
    }
}
