use get_if_addrs::{get_if_addrs, IfAddr};
use serde::{Deserialize, Serialize};
use std::error::Error;
use sysinfo::System;

pub mod resources;
pub mod storage;

pub use resources::Resources;

use crate::config::env::server_port;

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

fn get_computer_ip() -> Result<String, Box<dyn Error>> {
    let interfaces = get_if_addrs()?;
    for interface in interfaces {
        if let IfAddr::V4(addr) = interface.addr {
            return Ok(addr.ip.to_string());
        }
    }
    
    Ok("0.0.0.0".to_string())
}

impl ServerNode {
    pub fn new(id: u32) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            id,
            hostname: None,
            location: Location::IpAddress(IpAddress {
                address: get_computer_ip()?,
                port: server_port().parse::<u16>()?,
            }),
            status: ServerStatus::Online,
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
        let node = ServerNode::new(1).unwrap();

        assert_eq!(node.id, 1);
        assert_eq!(node.status, ServerStatus::Online);
        assert!(node.resources.cpus.len() > 0);
        assert!(node.system_info.name.len() > 0);
    }

    #[test]
    fn test_server_node_hostname() {
        let mut node = ServerNode::new(1).unwrap();
        node.hostname = Some("example.com".to_string());

        assert_eq!(node.hostname.unwrap(), "example.com");
    }
}
