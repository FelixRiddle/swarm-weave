use entity::{
	server_location::{
		ActiveModel as ServerLocationActiveModel, Entity as ServerLocationEntity,
		Model as ServerLocationModel,
	},
	server_node::Model as ServerNodeModel,
};
use get_if_addrs::{get_if_addrs, IfAddr};
use names::{Generator, Name};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, IntoActiveModel};
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::config::env::server_port;
use crate::database::mysql_connection;

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

#[derive(Clone)]
pub struct ServerInfoController {
	pub db: DatabaseConnection,
	pub server_info: ServerInfo,
}

impl ServerInfoController {
	pub async fn new() -> Result<Self, Box<dyn Error>> {
		let db = mysql_connection().await?;
		let server_info = ServerInfo::new()?;

		Ok(Self { db, server_info })
	}
	
	pub async fn insert(self) -> Result<ServerLocationModel, Box<dyn Error>> {
		let result = self
			.server_info
			.clone()
			.into_active_model()
			.insert(&self.db)
			.await?;

		Ok(result)
	}
	
	pub async fn update(self) -> Result<Self, Box<dyn Error>> {
		let updated_active_model = self.server_info
			.clone()
			.into_active_model()
			.save(&self.db)
			.await?;
		
		let updated_server_info = match ServerInfo::from_active_model(updated_active_model) {
			Some(server_info) => server_info,
			None => return Err("Failed to convert active model to server info".into()),
		};
		
		Ok(ServerInfoController {
			db: self.db,
			server_info: updated_server_info,
		})
	}
	
	/// Find by id
	/// 
	/// 
	pub async fn find(&mut self, id: i64) -> Result<&mut Self, Box<dyn Error>> {
		let found_server_info: Option<ServerLocationModel> =
			ServerLocationEntity::find_by_id(id).one(&self.db).await?;
		let server_info: ServerInfo = match found_server_info {
			Some(model) => model.into(),
			None => return Err("Server info not found".into()),
		};

		self.server_info = server_info;
		Ok(self)
	}
	
	/// Find by server node model
	/// 
	/// 
	pub async fn find_by_server_node_model(
		db: DatabaseConnection,
		server_node_active_model: ServerNodeModel
	) -> Result<ServerLocationModel, Box<dyn Error>> {
		let server_location_id = match server_node_active_model.server_location_id {
			Some(id) => id,
			None => return Err("Server location id not found".into()),
		};
		let server_location = match ServerLocationEntity::find_by_id(server_location_id).one(&db).await? {
			Some(model) => model,
            None => return Err("Server location not found".into()),
		};
		
		Ok(server_location)
	}
	
	pub async fn delete(self, id: i64) -> Result<Self, Box<dyn Error>> {
		ServerLocationEntity::delete_by_id(id)
			.exec(&self.db)
			.await?;

		Ok(self)
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

	#[tokio::test]
	async fn test_server_info_controller_new() {
		let controller = ServerInfoController::new().await.unwrap();
		assert!(controller.server_info.name.len() > 0);
	}

	#[tokio::test]
	async fn test_server_info_controller_insert() {
		let controller = ServerInfoController::new().await.unwrap();

		let inserted_model = controller.insert().await.unwrap();
		assert!(inserted_model.id > 0);
	}

	#[tokio::test]
	async fn test_server_info_controller_update() {
		let mut controller = ServerInfoController::new().await.unwrap();

		controller.clone().insert().await.unwrap();

		controller.server_info.name = "Updated Name".to_string();
		let mut active_model = controller.server_info.clone().into_active_model();
		active_model.name = ActiveValue::Set(controller.server_info.name.clone());

		let mut updated_model = active_model.save(&controller.db).await.unwrap();

		assert_eq!(updated_model.name.take().unwrap(), "Updated Name");
	}
	
	#[tokio::test]
	async fn test_server_info_controller_update_using_controller() {
		let mut controller = ServerInfoController::new().await.unwrap();
	
		controller.clone().insert().await.unwrap();
	
		controller.server_info.name = "Updated Name".to_string();
	
		let updated_controller = controller.update().await.unwrap();
	
		assert_eq!(updated_controller.server_info.name, "Updated Name");
	}
	
	#[tokio::test]
	async fn test_server_info_controller_find() {
		let mut controller = ServerInfoController::new().await.unwrap();

		let inserted_model = controller.clone().insert().await.unwrap();

		let found_controller = controller.find(inserted_model.id).await.unwrap();
		assert_eq!(found_controller.server_info.name, inserted_model.name);
	}

	#[tokio::test]
	async fn test_server_info_controller_delete() {
		let controller = ServerInfoController::new().await.unwrap();

		let inserted_model = controller.clone().insert().await.unwrap();

		let deleted_controller = controller.delete(inserted_model.id).await.unwrap();
		assert_eq!(deleted_controller.server_info.name, inserted_model.name);
	}
}
