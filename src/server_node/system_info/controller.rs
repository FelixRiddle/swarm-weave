use entity::{
	server_node::Model as ServerNodeModel,
	system_info::{
		ActiveModel as SystemInfoActiveModel, Entity as SystemInfoEntity, Model as SystemInfoModel,
	},
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use std::error::Error;

use super::SystemInfo;

#[derive(Clone)]
pub struct SystemInfoController {
	pub db: DatabaseConnection,
	pub system_info: SystemInfo,
	pub system_info_active_model: Option<SystemInfoActiveModel>,
}

/// System info controller
///
///
impl SystemInfoController {
	/// Create new system info controller
	///
	///
	pub async fn new(
		db: DatabaseConnection,
		system_info: SystemInfo,
	) -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			db: db.clone(),
			system_info,
			system_info_active_model: None,
		})
	}

	/// Create system info controller
	///
	/// Cloning a database connection is fine
	pub async fn new_bare(db: &DatabaseConnection) -> Result<Self, Box<dyn Error>> {
		let system_info = SystemInfo::new();

		Ok(Self {
			db: db.clone(),
			system_info,
			system_info_active_model: None,
		})
	}
}

/// Get
/// 
/// 
impl SystemInfoController {
	/// Id
	/// 
	/// 
	pub async fn id(&mut self) -> Result<i64, Box<dyn Error>> {
        let active_model = self.get_or_create_system_info().await?;
		
		let id = match active_model.id.clone().take() {
            Some(id) => id,
            None => return Err("System info id doesn't exists".into()),
        };
		
		Ok(id)
    }
	
	/// Get or create system info
	/// 
	/// 
	pub async fn get_or_create_system_info(&mut self) -> Result<SystemInfoActiveModel, Box<dyn Error>> {
		let active_model = match self.system_info_active_model.clone() {
			Some(system_info_active_model) => system_info_active_model,
			None => {
				// Create active model
				let mut active_model = self.system_info.clone().into_active_model();
				
				// Insert the model
				let new_model = active_model
					.clone()
					.insert(&self.db)
					.await?;
				
				// Update id
				active_model.id = ActiveValue::Unchanged(new_model.id.clone());
				
				// Update active model
				self.system_info_active_model = Some(active_model.clone());
				
				active_model
			}
		};
		
		Ok(active_model)
	}

	/// Find
	/// 
	/// 
	pub async fn find(&mut self, id: i64) -> Result<&mut Self, Box<dyn Error>> {
		let found_system_info: Option<SystemInfoModel> =
			SystemInfoEntity::find_by_id(id).one(&self.db).await?;
		
		// Convert to normal model
		let system_info: SystemInfo = match found_system_info {
			Some(model) => {
				// Update active model
				self.system_info_active_model = Some(model.clone().into());
				
				model.into()
			},
			None => return Err("System info not found".into()),
		};
		
		self.system_info = system_info;
		
		Ok(self)
	}
	
	/// Find by server node model
	///
	///
	pub async fn find_by_server_node_model(
		db: DatabaseConnection,
		server_node_model: ServerNodeModel,
	) -> Result<SystemInfoModel, Box<dyn Error>> {
		let system_info_id = match server_node_model.system_info_id {
			Some(id) => id,
			None => return Err("Server location id not found".into()),
		};
		let server_location = match SystemInfoEntity::find_by_id(system_info_id)
			.one(&db)
			.await?
		{
			Some(model) => model,
			None => return Err("Server location not found".into()),
		};

		Ok(server_location)
	}
}

impl SystemInfoController {
	/// Insert model
	/// 
	/// 
	pub async fn insert(&mut self) -> Result<&Self, Box<dyn Error>> {
		self.get_or_create_system_info().await?;
		
		Ok(self)
	}

	pub async fn update(self) -> Result<Self, Box<dyn Error>> {
		self.system_info
			.clone()
			.into_active_model()
			.update(&self.db)
			.await?;

		Ok(self)
	}

	pub async fn delete(self, id: i64) -> Result<Self, Box<dyn Error>> {
		SystemInfoEntity::delete_by_id(id).exec(&self.db).await?;

		Ok(self)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::database::mysql_connection;

	#[test]
	fn test_system_info_new() {
		let system_info = SystemInfo::new();
		assert!(system_info.name.len() > 0);
		assert!(system_info.kernel_version.len() > 0);
		assert!(system_info.os_version.len() > 0);
		assert!(system_info.host_name.len() > 0);
	}

	#[tokio::test]
	async fn test_system_info_controller_new() {
		let connection = mysql_connection().await.unwrap();
		let controller = SystemInfoController::new_bare(&connection).await.unwrap();
		assert!(controller.system_info.name.len() > 0);
	}

	#[tokio::test]
	async fn test_system_info_controller_insert() {
		let connection = mysql_connection().await.unwrap();
		
		// Create system info
		let mut controller = SystemInfoController::new_bare(&connection).await.unwrap();
		controller.insert().await.unwrap();
		
		// Get and validate system info
		let id = controller.id().await.unwrap();
		let mut new_controller = controller.clone();
		let found_controller = new_controller.find(id).await.unwrap();
		let id = found_controller.id().await.unwrap();
		
		assert!(found_controller.system_info.name.len() > 0);
		assert!(id > 0);
	}

	#[tokio::test]
	async fn test_system_info_controller_update() {
		let connection = mysql_connection().await.unwrap();
		let mut controller = SystemInfoController::new_bare(&connection).await.unwrap();

		controller.system_info.name = "New name".to_string();
		controller = controller.update().await.unwrap();
		assert_eq!(controller.system_info.name, "New name");
	}

	#[tokio::test]
	async fn test_system_info_controller_find() {
		let connection = mysql_connection().await.unwrap();
		let mut controller = SystemInfoController::new_bare(&connection).await.unwrap();

		// Insert model
		controller.insert().await.unwrap();
		
		// Find model
		let id = controller.id().await.unwrap();
		let mut new_controller = controller.clone();
		let found_controller = new_controller.find(id).await.unwrap();

		assert_eq!(
			found_controller.system_info.name,
			controller.system_info.name
		);
	}

	#[tokio::test]
	async fn test_system_info_controller_delete() {
		let connection = mysql_connection().await.unwrap();
		let mut controller = SystemInfoController::new_bare(&connection).await.unwrap();

		// Insert model
		controller.insert().await.unwrap();
		let id = controller.id().await.unwrap();
		
		// Delete data
		let mut controller = controller.delete(id).await.unwrap();
		
		assert!(controller.find(id).await.is_err());
	}
}
