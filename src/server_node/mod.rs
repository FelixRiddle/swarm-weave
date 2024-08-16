use names::{Generator, Name};
use get_if_addrs::{get_if_addrs, IfAddr};
use serde::{Deserialize, Serialize};
use std::error::Error;
use sysinfo::System;

pub mod resources;
pub mod storage;

pub use resources::Resources;

use crate::config::env::server_port;

#[derive(Deserialize, Serialize)]
pub struct IpAddress {
    pub address: String,
    pub port: u16,
}

impl IpAddress {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            address: get_computer_ip()?,
            port: server_port().parse::<u16>()?,
        })
    }
}

#[derive(Deserialize, Serialize)]
pub enum ServerLocation {
    IpAddress(IpAddress),
    DomainName(String),
}

#[derive(Deserialize, Serialize)]
pub struct ServerInfo {
    // Display name
    pub name: String,
    pub hostname: Option<String>,
    pub location: ServerLocation,
}

impl ServerInfo {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut generator = Generator::with_naming(Name::Numbered);
        let name = match generator.next() {
            Some(name) => name,
            None => return Err("Failed to generate a unique name for the server".into()),
        };
        
        let hostname = None;
        let ip_address = IpAddress::new()?;
        let location = ServerLocation::IpAddress(ip_address);
        
        Ok(Self {
            name,
            hostname,
            location,
        })
    }
}

#[derive(Deserialize, Serialize)]
pub struct ServerNode {
    pub id: u32,
    pub location: ServerInfo,
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
            let ip = addr.ip;
            if !ip.is_loopback() {
                return Ok(ip.to_string());
            }
        }
    }
    
    Ok("0.0.0.0".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info_new() {
        let server_info = ServerInfo::new().unwrap();
        assert!(server_info.name.len() > 0);
        assert!(server_info.hostname.is_none());
        match server_info.location {
            ServerLocation::IpAddress(ip_address) => {
                assert!(ip_address.address.len() > 0);
                assert!(ip_address.port > 0);
            }
            ServerLocation::DomainName(_) => panic!("Unexpected DomainName"),
        }
    }

    #[test]
    fn test_server_node_new() {
        let server_node = ServerNode::new(1).unwrap();
        assert!(server_node.id > 0);
        assert!(server_node.location.name.len() > 0);
        assert_eq!(server_node.status, ServerStatus::Online);
        assert!(server_node.system_info.name.len() > 0);
    }

    #[test]
    fn test_system_info_new() {
        let system_info = SystemInfo::new();
        assert!(system_info.name.len() > 0);
        assert!(system_info.kernel_version.len() > 0);
        assert!(system_info.os_version.len() > 0);
        assert!(system_info.host_name.len() > 0);
    }

    #[test]
    fn test_get_computer_ip() {
        let ip = get_computer_ip().unwrap();
        assert!(ip.len() > 0);
    }
}
