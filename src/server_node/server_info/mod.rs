use entity::server_location::{
	ActiveModel as ServerLocationActiveModel,
	Model as ServerLocationModel,
};
use get_if_addrs::{get_if_addrs, IfAddr};
use names::{Generator, Name};
use sea_orm::{ActiveValue, IntoActiveModel};
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::config::env::server_port;

pub mod controller;

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
	// TODO: Location can be any of the two
	//pub domain: Option<String>,
	//pub address: Option<IpAddress>,
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
	
	/// Create from model
	/// 
	/// 
	pub fn from_model(model: ServerLocationModel) -> Option<Self> {
		let model = model.into_active_model();
		let model = Self::from_active_model(model);
		
		model
	}
	
	/// Create from active model
	/// 
	/// 
	pub fn from_active_model(active_model: ServerLocationActiveModel) -> Option<Self> {
		let mut active_model = active_model.clone();
		
		let name = match active_model.name.take() {
			Some(name) => name,
			None => return None,
		};
		
		let hostname = match active_model.domain.take() {
			Some(hostname) => hostname,
			None => None,
		};
		
		let location = match (active_model.domain.take(), active_model.address.take(), active_model.port.take()) {
			(Some(domain_name), _, _) => {
				match domain_name {
					Some(domain_name) => Some(ServerLocation::DomainName(domain_name)),
					None => None
				}
			},
			(_, Some(address), Some(port)) => Some(ServerLocation::IpAddress(IpAddress {
				address: address.unwrap(),
				port: port.unwrap() as u16,
			})),
			_ => return None,
		};
		
		let location = match location {
			Some(location) => location,
			None => return None,
		};
		
		Some(Self {
			name,
			hostname,
			location,
		})
	}
}

impl Into<ServerInfo> for ServerLocationModel {
	fn into(self) -> ServerInfo {
		ServerInfo {
			name: self.name,
			hostname: self.domain.clone(),
			location: match self.domain {
				Some(domain_name) => ServerLocation::DomainName(domain_name),
				None => match (self.address, self.port) {
					(Some(address), Some(port)) => ServerLocation::IpAddress(IpAddress {
						address,
						port: port as u16,
					}),
					_ => panic!("Invalid server location"),
				},
			},
		}
	}
}

/// Conversions
/// 
/// 
impl ServerInfo {
	/// Convert into active model
	/// 
	/// 
	pub fn into_active_model(&self) -> ServerLocationActiveModel {
		ServerLocationActiveModel {
			name: ActiveValue::Set(self.name.clone()),
			domain: ActiveValue::Set(self.hostname.clone()),
			address: ActiveValue::Set(match &self.location {
				ServerLocation::IpAddress(ip_address) => Some(ip_address.address.clone()),
				ServerLocation::DomainName(domain_name) => Some(domain_name.clone()),
			}),
			port: ActiveValue::Set(match &self.location {
				ServerLocation::IpAddress(ip_address) => Some(ip_address.port as i32),
				ServerLocation::DomainName(_) => None,
			}),
			..Default::default()
		}
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
