//! This module implements a centralized service using libp2p.
//! 
//! It combines Gossipsub and mDNS to enable peer discovery and message propagation.
use std::error::Error;
use super::HiveParameters;

use crate::p2p::node::Node;

/// Start service
/// 
/// 
pub async fn main(parameters: HiveParameters) -> Result<(), Box<dyn Error>> {
    let mut node = Node::new(parameters).await?;
    
    node.start().await?;
    
    Ok(())
}
