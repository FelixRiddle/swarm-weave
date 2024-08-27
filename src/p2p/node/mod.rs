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
use crate::test::folder::hive_folder_test_suite::hive_server_node::HiveServerNode;

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
    // Test handler to manage testing information
    pub test_handler: Option<HiveServerNode>,
}

impl Node {
    pub async fn new(parameters: HiveParameters) -> Result<Self, Box<dyn Error>> {
        // Fetch key seed
        let key_seed = match parameters.key_seed {
            Some(seed) => seed,
            None => return Err("Key seed is required".into()),
        };
        
        // Logger
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
            test_handler: None,
        })
    }
    
    /// New with test handler
    /// 
    /// 
    pub async fn new_with_test_handler(
        parameters: HiveParameters,
        test_handler: HiveServerNode,
    ) -> Result<Self, Box<dyn Error>> {
        // Fetch key seed
        let key_seed = match parameters.key_seed {
            Some(seed) => seed,
            None => return Err("Key seed is required".into()),
        };
        
        // In tests this fails, because there are two global subscribers set
        // if let Some(subscriber) = tracing::subscriber::global_default() {
        //     // A subscriber is already set, handle it or exit
        // } else {
        //     // No subscriber is set, set one or handle the case
        // }
        
        // Logger, because we know we have a test handler, we want to write logs to a folder
        let file_appender = tracing_appender::rolling::daily(test_handler.get_log_folder(), "log");
        let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_writer(non_blocking_writer)
            .init();
        
        // Create swarm
        let local_key: identity::Keypair = generate_ed25519(key_seed)?;
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
        
        // Create and subscribe to topics
        let topic = gossipsub::IdentTopic::new("chat-net");
        swarm.behaviour_mut()
            .gossipsub.subscribe(&topic)?;
        swarm.behaviour_mut()
            .gossipsub.subscribe(&gossipsub::IdentTopic::new("test-chat"))?;
        
        Ok(Node {
            parameters,
            local_key,
            swarm,
            topic,
            test_handler: None,
        })
    }
    
    /// Set test handler
    /// 
    /// 
    pub fn set_test_handler(&mut self, test_handler: HiveServerNode) {
        self.test_handler = Some(test_handler);
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let port = self.parameters.get_port();
        
        // Relay
        if self.parameters.relay {
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
        }
        
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
                                MyBehaviorEvent::Gossipsub(event) => {
                                    match event {
                                        gossipsub::Event::Message {
                                            propagation_source: peer_id,
                                            message_id: id,
                                            message,
                                        } => {
                                            let topic = message.topic;
                                            let topic_name = topic.to_string();
                                            let topic_name = topic_name.as_str();
                                            
                                            match topic_name {
                                                "chat-net" => {
                                                    println!(
                                                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                                                        String::from_utf8_lossy(&message.data),
                                                    );
                                                }
                                                "test-chat" => {
                                                    println!("test-chat: {}", String::from_utf8_lossy(&message.data));
                                                }
                                                _ => {}
                                            };
                                        }
                                        _ => {}
                                    };
                                }
                                // Add relay nodes
                                MyBehaviorEvent::Identify(event) => {
                                    match event {
                                        identify::Event::Received {
                                            info,
                                            ..
                                        } => {
                                            let observed_addr = info.observed_addr;
                                            
                                            // If we're a relay node, add the peer's address to our swarm
                                            // If we're not a relay node, we don't need to do anything here
                                            if self.parameters.relay {
                                                self.swarm.add_external_address(observed_addr.clone());
                                            }
                                        }
                                        _ => { }
                                    };
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

/// Really hard to test
/// 
/// 
#[cfg(test)]
mod tests {
    // use std::str::FromStr;

    use super::*;
    // use libp2p::identity::Keypair;
    // use libp2p::swarm::Swarm;
    // use libp2p::PeerId;
    // use libp2p::gossipsub::IdentTopic;
    // use libp2p::mdns::Event;
    // use libp2p::tcp::Config as TcpConfig;
    // use libp2p::yamux::Config as YamuxConfig;
    // use libp2p::noise::Config as NoiseConfig;
    // use libp2p::Multiaddr;
    // use std::time::Duration;
    
    use crate::test::folder::hive_folder_test_suite::HiveFolderTestSuite;
    
    #[tokio::test]
    async fn test_new_node() {
        let suite = HiveFolderTestSuite::default();
        let test_node = suite.create_server_node("127.0.0.1".to_string(), 45829).unwrap();
        
        let parameters = HiveParameters {
            server: true,
            key_seed: Some(120),
            relay: true,
            use_ipv6: Some(false),
            port: Some(45829),
            server_address: None,
            server_peer_id: None,
        };
        
        let result = Node::new(parameters).await;
        assert!(result.is_ok());
        
        test_node.save_config().unwrap();
    }
    
    #[tokio::test]
    async fn test_new_node_with_test_handler() {
        let parameters = HiveParameters {
            server: true,
            key_seed: Some(120),
            relay: true,
            use_ipv6: Some(false),
            port: Some(45830),
            server_address: None,
            server_peer_id: None,
        };
        
        let suite = HiveFolderTestSuite::default();
        let test_handler = suite.create_server_node("127.0.0.1".to_string(), 45830).unwrap();
        
        let result = Node::new_with_test_handler(parameters, test_handler).await;
        assert!(result.is_ok());
        
        let node = result.unwrap();
        assert_eq!(node.parameters.key_seed, Some(120));
        assert_eq!(node.parameters.relay, true);
        assert_eq!(node.parameters.use_ipv6, Some(false));
        assert_eq!(node.parameters.port, Some(45830));
        assert!(node.test_handler.is_some());
        
        node.test_handler.unwrap().save_config().unwrap();
    }
    
    // TODO: Test that the chat works, by starting two nodes and sending a private key or something
    
    // TODO: Test that the relay works
}
