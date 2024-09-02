use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub mod server_info;
pub mod system_info;
pub mod resources;
pub mod storage;

pub use resources::Resources;
pub use system_info::SystemInfo;
pub use server_info::ServerInfo;

#[derive(Debug, PartialEq)]
#[derive(Deserialize, Serialize)]
pub enum ServerStatus {
    Online,
    Offline,
    Maintenance,
}

#[derive(Deserialize, Serialize)]
pub struct ServerNode {
    pub id: u32,
    pub location: ServerInfo,
    pub status: ServerStatus,
    pub resources: Resources,
    pub system_info: SystemInfo,
}

impl ServerNode {
    pub fn new(id: u32) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            id,
            location: ServerInfo::new()?,
            status: ServerStatus::Online,
            resources: Resources::fetch_resources()?,
            system_info: SystemInfo::new(),
        })
    }
}

/// Server node controller
/// 
/// Mainly for database manipulation
pub struct ServerNodeController {
	pub db: DatabaseConnection,
	pub server_node: ServerNode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_node_new() {
        let server_node = ServerNode::new(1).unwrap();
        assert!(server_node.id > 0);
        assert!(server_node.location.name.len() > 0);
        assert_eq!(server_node.status, ServerStatus::Online);
        assert!(server_node.system_info.name.len() > 0);
    }
}
