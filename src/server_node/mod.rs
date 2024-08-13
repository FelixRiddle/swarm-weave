use serde::{Deserialize, Serialize};
use std::error::Error;
use sysinfo::System;

pub mod resources;
pub mod storage;

pub use resources::Resources;

#[derive(Deserialize, Serialize)]
pub enum Location {
    IpAddress(IpAddress),
    DomainName(String),
}

#[derive(Deserialize, Serialize)]
pub struct IpAddress {
    pub address: String,
    pub port: u16,
}

#[derive(Deserialize, Serialize)]
pub struct ServerNode {
    pub id: u32,
    pub hostname: Option<String>,
    pub location: Location,
    pub status: ServerStatus,
    pub resources: Resources,
    pub system_info: SystemInfo,
}

#[derive(Debug, PartialEq)]
#[derive(Deserialize, Serialize)]
pub enum ServerStatus {
    Online,
    Offline,
    Maintenance,
}

#[derive(Deserialize, Serialize)]
pub struct SystemInfo {
    pub name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub host_name: String,
}

impl SystemInfo {
    pub fn new() -> Self {
        Self {
            name: System::name().unwrap_or("Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or("Unknown".to_string()),
            os_version: System::os_version().unwrap_or("Unknown".to_string()),
            host_name: System::host_name().unwrap_or("Unknown".to_string()),
        }
    }
}

impl ServerNode {
    pub fn new(id: u32, location: Location, status: ServerStatus) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            id,
            hostname: None,
            location,
            status,
            resources: Resources::fetch_resources()?,
            system_info: SystemInfo::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_node_new() {
        let location = Location::IpAddress(IpAddress {
            address: "127.0.0.1".to_string(),
            port: 8080,
        });
        let node = ServerNode::new(1, location, ServerStatus::Online).unwrap();

        assert_eq!(node.id, 1);
        assert_eq!(node.status, ServerStatus::Online);
        assert!(node.resources.cpus.len() > 0);
        assert!(node.system_info.name.len() > 0);
    }

    #[test]
    fn test_server_node_hostname() {
        let location = Location::IpAddress(IpAddress {
            address: "127.0.0.1".to_string(),
            port: 8080,
        });
        let mut node = ServerNode::new(1, location, ServerStatus::Online).unwrap();
        node.hostname = Some("example.com".to_string());

        assert_eq!(node.hostname.unwrap(), "example.com");
    }
}
