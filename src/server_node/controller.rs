use entity::server_node::{ActiveModel as ServerNodeActiveModel, Entity as ServerNodeEntity};
use entity::{
	server_location::ActiveModel as ServerLocationActiveModel,
	system_info::ActiveModel as SystemInfoActiveModel,
	system_resources::ActiveModel as SystemResourcesActiveModel,
};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::error::Error;

use super::resources::controller::SystemResourcesController;
use super::server_info::{
	ServerInfo,
	ServerInfoController,
};
use super::system_info::{
	SystemInfo,
	SystemInfoController,
};
use super::{
	ServerStatus,
	ServerNode,
};

/// Server node controller
///
/// Mainly for database manipulation
pub struct ServerNodeController {
	pub db: DatabaseConnection,
	pub server_node: Option<ServerNode>,
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
		server_node: Option<ServerNode>,
		server_node_active_model: Option<ServerNodeActiveModel>,
		server_location: ServerLocationActiveModel,
		system_resources: SystemResourcesActiveModel,
		system_info: SystemInfoActiveModel,
	) -> Result<Self, Box<dyn Error>> {
		
		Ok(Self {
			db,
			server_node,
			server_node_active_model,
			server_location,
			system_resources,
			system_info,
		})
	}
	
	/// Get server node
	/// 
	/// The server node is cloned
	/// If server node doesn't exists create it
	pub fn get_server_node(&self) -> Result<ServerNode, Box<dyn Error>> {
		let server_node = match self.server_node.clone() {
			Some(server_node) => server_node,
			None => {
				let server_node = ServerNode::new(1)?;
				server_node
			}
		};
		
		Ok(server_node)
	}
	
	/// Create server node from id
	/// 
	/// 
	pub async fn server_node_from_id(
		db: DatabaseConnection,
		id: u32
	) -> Result<ServerNode, Box<dyn Error>> {
		// Find server node id
		let server_node_active_model = ServerNodeEntity::find_by_id(id).one(&db).await?;
		let server_node = match server_node_active_model {
			Some(server_node_active_model) => {
				// Take status
				let status = server_node_active_model.status.clone();
				let server_node_id = server_node_active_model.id.clone();
				
				// Find server location
				let server_location_model = ServerInfoController::find_by_server_node_model(db.clone(), server_node_active_model.clone())
					.await?;
				let server_location = match ServerInfo::from_model(server_location_model.clone()) {
					Some(server_location) => server_location,
					None => return Err("Couldn't convert server location model to server info".into()),
				};
				
				// Find system resources
				let system_resources_model = SystemResourcesController::find_by_server_node_model(db.clone(), server_node_active_model.clone())
					.await?;
				
				// Take id
				let system_resources_id = system_resources_model.id.clone();
				
				// Create resources object from models
				let system_resources_controller = SystemResourcesController::new(db.clone(), None);
				let resources = system_resources_controller.find_by_id_and_get_resources(
					system_resources_model,
					system_resources_id,
				).await?;
				
				// Find system info
				let system_info_model = SystemInfoController::find_by_server_node_model(db.clone(), server_node_active_model)
					.await?;
				let system_info = match SystemInfo::from_model(system_info_model.clone()) {
					Some(system_info) => system_info,
                    None => return Err("Couldn't convert system info model to system info".into()),
				};
				
				// Serve status
				let status: ServerStatus = match status {
					Some(status) => ServerStatus::from_status(status),
                    None => ServerStatus::Offline,
				};
				
				// Create server node
				let server_node = ServerNode {
					id: server_node_id.try_into()?,
					location: server_location,
					status,
					resources,
					system_info,
				};
				
				server_node
			},
			None => return Err("Server node not found".into()),
		};
		
		Ok(server_node)
	}

	// /// TODO: Create new from server node id
	// /// 
	// /// 
	// pub async fn new_from_server_node(
	// 	db: DatabaseConnection,
	// 	id: u32
	// ) -> Result<Self, Box<dyn Error>> {
	// 	let server_node = ServerNodeController::server_node_from_id(db, id)
	// 		.await?;
		
	// 	Ok(Self {
	// 		db,
	// 		server_node,
	// 		server_node_active_model: Some(server_node_active_model),
	// 		server_location: server_location_model.into_active_model(),
	// 		system_resources: system_resources.into_active_model(),
	// 		system_info: system_info.into_active_model(),
	// 	})
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
				let model = self
					.get_server_node()?
					.try_into_active_model(
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
