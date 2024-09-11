use entity::server_node::{ActiveModel as ServerNodeActiveModel, Entity as ServerNodeEntity};
use entity::{
	sea_orm_active_enums::Status,
	server_location::{
		ActiveModel as ServerLocationActiveModel,
		Entity as ServerLocationEntity,
	},
	system_info::ActiveModel as SystemInfoActiveModel,
	system_resources::ActiveModel as SystemResourcesActiveModel,
};
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, IntoActiveModel};
use serde::{Deserialize, Serialize};
use std::error::Error;
use strum_macros::Display;

pub mod resources;
pub mod server_info;
pub mod storage;
pub mod system_info;

pub use resources::{
	Resources,
	controller::SystemResourcesController,
	system_core::controller::CpuCoreController,
};
pub use server_info::{
	ServerInfo,
	ServerInfoController,
};
pub use system_info::{
	SystemInfo,
	SystemInfoController,
};

#[derive(Clone, Debug, Display, PartialEq, Deserialize, Serialize)]
pub enum ServerStatus {
	Online,
	Offline,
	Maintenance,
}

impl From<ServerStatus> for Status {
	fn from(status: ServerStatus) -> Self {
		match status {
			ServerStatus::Online => Status::Online,
			ServerStatus::Offline => Status::Offline,
			ServerStatus::Maintenance => Status::Maintenance,
		}
	}
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ServerNode {
	pub id: u32,
	pub location: ServerInfo,
	pub status: ServerStatus,
	pub resources: Resources,
	pub system_info: SystemInfo,
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

	/// Try into active model
	///
	///
	pub fn try_into_active_model(
		self,
		server_location_id: i64,
		resource_id: i64,
		system_info_id: i64,
	) -> Result<ServerNodeActiveModel, Box<dyn Error>> {
		Ok(ServerNodeActiveModel {
			id: ActiveValue::Set(i64::try_from(self.id)?),
			status: ActiveValue::Set(Some(self.status.into())),
			server_location_id: ActiveValue::Set(Some(server_location_id)),
			system_resource_id: ActiveValue::Set(Some(resource_id)),
			system_info_id: ActiveValue::Set(Some(system_info_id)),
			..Default::default()
		})
	}
}

/// Server node controller
///
/// Mainly for database manipulation
pub struct ServerNodeController {
	pub db: DatabaseConnection,
	pub server_node: ServerNode,
	server_node_active_model: Option<ServerNodeActiveModel>,
	// Models
	pub server_location: ServerLocationActiveModel,
	pub system_resources: SystemResourcesActiveModel,
	pub system_info: SystemInfoActiveModel,
}

impl ServerNodeController {
	/// Create new
	///
	/// Fetch resources on creation
	pub async fn new(
		db: DatabaseConnection,
		server_node_active_model: Option<ServerNodeActiveModel>,
		server_location: ServerLocationActiveModel,
		system_resources: SystemResourcesActiveModel,
		system_info: SystemInfoActiveModel,
	) -> Result<Self, Box<dyn Error>> {
		let server_node = ServerNode::new(1)?;
		
		Ok(Self {
			db,
			server_node,
			server_node_active_model,
			server_location,
			system_resources,
			system_info,
		})
	}

	// /// TODO: Create new from server node id
	// /// 
	// /// 
	// pub async fn new_from_server_node(
	// 	db: DatabaseConnection,
	// 	id: u32
	// ) -> Result<Self, Box<dyn Error>> {
	// 	// Find server node id
	// 	let server_node_active_model = ServerNodeEntity::find_by_id(id).one(&db).await?;
	// 	match server_node_active_model {
	// 		Some(server_node_active_model) => {
	// 			// Find server location
	// 			let server_location_model = ServerInfoController::find_by_server_node_model(db.clone(), server_node_active_model)
	// 				.await?;
	// 			let server_location = match ServerInfo::from_model(server_location_model.clone()) {
	// 				Some(server_location) => server_location,
	// 				None => return Err("Couldn't convert server location model to server info".into()),
	// 			};
				
	// 			// Find system resources
	// 			let system_resources_model = SystemResourcesController::find_by_server_node_model(db.clone(), server_node_active_model)
	// 				.await?;
				
	// 			// Now I need to find resources submodels
	// 			let cpu_core_controller = CpuCoreController::new(
	// 				db.clone(),
	// 				None,
	// 				None
	// 			);
	// 			let cpu_cores = cpu_core_controller.find_cores_by_resources_id(system_resources_model.id).await?;
				
	// 			// TODO: Find memory
	// 			// TODO: Find storage devices
				
	// 			let system_resources = match Resources::from_models(
	// 				system_resources_model.clone(),
					
	// 			) {
	// 				Some(system_resources) => system_resources,
	// 				None => return Err("Couldn't convert system resources model to system resources".into())
	// 			};
				
	// 			// Find system info
	// 			let system_info_model = SystemInfoController::find_by_server_node_model(db.clone(), server_node_active_model)
	// 				.await?;
	// 			let system_info = match SystemInfo::from_model(system_info_model.clone()) {
	// 				Some(system_info) => system_info,
    //                 None => return Err("Couldn't convert system info model to system info".into()),
	// 			};
				
	// 			// TODO: Create server node
	// 			let server_node = ServerNode {
	// 				id: server_node_active_model.id.try_into()?,
	// 				location: server_location,
	// 				status: server_node_active_model.status.into(),
	// 				resources: Resources::from_active_model(system_resources)?,
	// 				system_info,
	// 			};
				
	// 			Ok(Self {
	// 				db,
	// 				server_node,
	// 				server_node_active_model: Some(server_node_active_model),
	// 				server_location: server_location_model.into_active_model(),
	// 				system_resources: system_resources.into_active_model(),
	// 				system_info: system_info.into_active_model(),
	// 			})
	// 		},
	// 		None => Err("Server node not found".into()),
	// 	}
	// }

	/// Get or create server node active model
	///
	/// On creation the server node will be inserted, to make things faster
	pub async fn get_server_node_active_model(
		&mut self,
	) -> Result<ServerNodeActiveModel, Box<dyn Error>> {
		let model = match &self.server_node_active_model {
			Some(model) => model.clone(),
			None => {
				let server_location_id = match self.server_location.id.clone().take() {
					Some(id) => id,
					None => return Err("Server location id is not provided".into()),
				};
				let system_resource_id = match self.system_resources.id.clone().take() {
					Some(id) => id,
					None => return Err("System resource id is not provided".into()),
				};
				let system_info_id = match self.system_info.id.clone().take() {
					Some(id) => id,
					None => return Err("System info id is not provided".into()),
				};
				let model = self.server_node.clone().try_into_active_model(
					server_location_id,
					system_resource_id,
					system_info_id,
				)?;

				self.server_node_active_model = Some(model.clone());

				model
			}
		};

		Ok(model)
	}

	/// Insert
	///
	///
	pub async fn insert(&mut self) -> Result<&mut Self, Box<dyn Error>> {
		self.get_server_node_active_model().await?;
		Ok(self)
	}

	/// Delete
	///
	///
	pub async fn delete(&mut self) -> Result<&mut Self, Box<dyn Error>> {
		let server_node_active_model = self.get_server_node_active_model().await?;
		let id = match server_node_active_model.id.try_as_ref() {
			Some(id) => id,
			None => return Err("Server node id doesn't exists".into()),
		};
		ServerNodeEntity::delete_by_id(id.clone())
			.exec(&self.db)
			.await?;

		Ok(self)
	}

	/// Delete by id
	///
	///
	pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> Result<(), Box<dyn Error>> {
		ServerNodeEntity::delete_by_id(id).exec(db).await?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_server_node_new() {
		let server_node = ServerNode::new(1).unwrap();
		assert!(server_node.id > 0);
		assert!(server_node.location.name.len() > 0);
		assert_eq!(server_node.status, ServerStatus::Online);
		assert!(server_node.system_info.name.len() > 0);
	}

	/// Test serialization
	///
	///
	#[test]
	fn test_server_status_serialization() {
		let status = ServerStatus::Online;
		let serialized_status = serde_json::to_string(&status).unwrap();

		// It looks like this, not what I expected
		assert_eq!("\"Online\"", serialized_status);
	}

	/// Test to string
	///
	///
	#[test]
	fn test_server_status_to_string() {
		let status = ServerStatus::Online;
		let serialized_status = status.to_string();

		assert_eq!("Online", serialized_status);
	}
}
