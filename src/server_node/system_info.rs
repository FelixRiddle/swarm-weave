use entity::{
	system_info::{
		Entity as SystemInfoEntity,
		ActiveModel as SystemInfoActiveModel,
		Model as SystemInfoModel,
	},
	server_node::Model as ServerNodeModel,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, IntoActiveModel};
use serde::{Deserialize, Serialize};
use std::error::Error;
use sysinfo::System;

#[derive(Clone, Deserialize, Serialize)]
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

/// Transformations
/// 
/// 
impl SystemInfo {	
	/// Convert into active model
	/// 
	/// 
	pub fn into_active_model(self) -> SystemInfoActiveModel {
		SystemInfoActiveModel {
			name: ActiveValue::Set(self.name),
			kernel_version: ActiveValue::Set(Some(self.kernel_version)),
			os_version: ActiveValue::Set(self.os_version),
			hostname: ActiveValue::Set(self.host_name),
			..Default::default()
		}
	}
	
	/// Create from model
	/// 
	/// 
	pub fn from_model(model: SystemInfoModel) -> Option<Self> {
		let model = model.into_active_model();
		let model = Self::from_active_model(model);
		
		model
	}
	
	/// Create from active model
	/// 
	/// 
	pub fn from_active_model(active_model: SystemInfoActiveModel) -> Option<Self> {
		let name = active_model.name.clone().take()?;
		
		let kernel_version = active_model.kernel_version.clone().take();
		let kernel_version = match kernel_version {
			Some(kernel_version) => {
				let kernel_version = match kernel_version {
					Some(kernel_version) => kernel_version,
					None => String::from("Unknown")
				};
				
				kernel_version
			},
			None => return None,
		};
		
		let os_version = active_model.os_version.clone().take()?;
		let host_name = active_model.hostname.clone().take()?;
		
		Some(Self {
			name,
			kernel_version,
			os_version,
			host_name,
		})
	}
}

impl Into<SystemInfo> for SystemInfoModel {
	fn into(self) -> SystemInfo {
		SystemInfo {
			name: self.name,
			kernel_version: self.kernel_version.unwrap_or("Unknown".to_string()),
			os_version: self.os_version,
			host_name: self.hostname,
		}
	}
}

#[derive(Clone)]
pub struct SystemInfoController {
	pub db: DatabaseConnection,
	pub system_info: SystemInfo,
}

impl SystemInfoController {
	/// Create system info controller
	/// 
	/// Cloning a database connection is fine
	pub async fn new(db: &DatabaseConnection) -> Result<Self, Box<dyn Error>> {
		let system_info = SystemInfo::new();
		
		Ok(Self { db: db.clone(), system_info })
	}
	
	pub async fn insert(self) -> Result<SystemInfoModel, Box<dyn Error>> {
		let result = self.system_info.clone().into_active_model().insert(&self.db).await?;
		
		Ok(result)
	}
	
	pub async fn update(self) -> Result<Self, Box<dyn Error>> {
		self.system_info.clone().into_active_model().update(&self.db).await?;
		
		Ok(self)
	}
	
	pub async fn find(&mut self, id: i64) -> Result<&mut Self, Box<dyn Error>> {
		let found_system_info: Option<SystemInfoModel> = SystemInfoEntity::find_by_id(id).one(&self.db).await?;
		let system_info: SystemInfo = match found_system_info {
			Some(model) => model.into(),
			None => return Err("System info not found".into()),
		};
		
		self.system_info = system_info;
		Ok(self)
	}
	
	pub async fn delete(self, id: i64) -> Result<Self, Box<dyn Error>> {
		let delete_result = SystemInfoEntity::delete_by_id(id).exec(&self.db).await?;
		assert_eq!(delete_result.rows_affected, 1);
		
		Ok(self)
	}
	
	/// Find by server node model
	/// 
	/// 
	pub async fn find_by_server_node_model(
		db: DatabaseConnection,
		server_node_active_model: ServerNodeModel
	) -> Result<SystemInfoModel, Box<dyn Error>> {
		let system_info_id = match server_node_active_model.system_info_id {
			Some(id) => id,
			None => return Err("Server location id not found".into()),
		};
		let server_location = match SystemInfoEntity::find_by_id(system_info_id).one(&db).await? {
			Some(model) => model,
			None => return Err("Server location not found".into()),
		};
		
		Ok(server_location)
	}
}

#[cfg(test)]
pub mod tests {
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
		let controller = SystemInfoController::new(&connection).await.unwrap();
		assert!(controller.system_info.name.len() > 0);
	}

	#[tokio::test]
	async fn test_system_info_controller_insert() {
		let connection = mysql_connection().await.unwrap();
		let controller = SystemInfoController::new(&connection).await.unwrap();
		let model = controller.clone().insert().await.unwrap();
		assert!(model.name.len() > 0);
		assert!(model.id > 0);
	}

	#[tokio::test]
	async fn test_system_info_controller_update() {
		let connection = mysql_connection().await.unwrap();
		let mut controller = SystemInfoController::new(&connection).await.unwrap();
		
		controller.system_info.name = "New name".to_string();
		controller = controller.update().await.unwrap();
		assert_eq!(controller.system_info.name, "New name");
	}

	#[tokio::test]
	async fn test_system_info_controller_find() {
		let connection = mysql_connection().await.unwrap();
		let controller = SystemInfoController::new(&connection).await.unwrap();
		
		// Insert model
		let model = controller.clone().insert().await.unwrap();
		
		// Find model
		let mut new_controller = controller.clone();
		let found_controller = new_controller.find(model.id).await.unwrap();
		
		assert_eq!(found_controller.system_info.name, controller.system_info.name);
	}

	#[tokio::test]
	async fn test_system_info_controller_delete() {
		let connection = mysql_connection().await.unwrap();
		let controller = SystemInfoController::new(&connection).await.unwrap();
		
		let model = controller.clone().insert().await.unwrap();
		let mut controller = controller.delete(model.id).await.unwrap();
		assert!(controller.find(model.id).await.is_err());
	}
}
