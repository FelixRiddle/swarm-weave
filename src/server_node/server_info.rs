use names::{Generator, Name};
use get_if_addrs::{get_if_addrs, IfAddr};
use serde::{Deserialize, Serialize};
use std::error::Error;
use entity::server_location::{
    Entity as ServerLocationEntity,
    ActiveModel as ServerLocationActiveModel,
    Model as ServerLocationModel,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};

use crate::database::mysql_connection;
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

impl ServerInfo {
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
        let result = self.server_info.clone()
			.into_active_model()
			.insert(&self.db)
			.await?;
        
        Ok(result)
    }
    
    pub async fn update(self) -> Result<Self, Box<dyn Error>> {
        self.server_info.clone()
			.into_active_model()
			.update(&self.db)
			.await?;
        
        Ok(self)
    }
    
    pub async fn find(&mut self, id: i64) -> Result<&mut Self, Box<dyn Error>> {
        let found_server_info: Option<ServerLocationModel> = ServerLocationEntity::find_by_id(id).one(&self.db).await?;
        let server_info: ServerInfo = match found_server_info {
            Some(model) => model.into(),
            None => return Err("Server info not found".into()),
        };
        
        self.server_info = server_info;
        Ok(self)
    }
    
    pub async fn delete(self, id: i64) -> Result<Self, Box<dyn Error>> {
        let delete_result = ServerLocationEntity::delete_by_id(id).exec(&self.db).await?;
        assert_eq!(delete_result.rows_affected, 1);
        
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
