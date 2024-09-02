use names::{Generator, Name};
use get_if_addrs::{get_if_addrs, IfAddr};
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::config::env::server_port;

/// Get computer IP v4
/// 
/// 
pub fn get_computer_ip() -> Result<String, Box<dyn Error>> {
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

#[derive(Clone, Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
pub enum ServerLocation {
    IpAddress(IpAddress),
    DomainName(String),
}

#[derive(Clone, Deserialize, Serialize)]
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

#[cfg(test)]
pub mod tests {
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
    fn test_get_computer_ip() {
        let ip = get_computer_ip().unwrap();
        assert!(ip.len() > 0);
    }
}
