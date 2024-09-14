use entity::{
	server_location::{
		Entity as ServerLocationEntity,
		Model as ServerLocationModel,
	},
	server_node::Model as ServerNodeModel,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use std::error::Error;

use crate::database::mysql_connection;
use super::ServerInfo;

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
	use super::super::{
		*
	};
	
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
