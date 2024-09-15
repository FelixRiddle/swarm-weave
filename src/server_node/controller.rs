use entity::server_node::{ActiveModel as ServerNodeActiveModel, Entity as ServerNodeEntity};
use entity::{
	server_location::{
		ActiveModel as ServerLocationActiveModel,
		Model as ServerLocationModel,
	},
	system_info::{
		ActiveModel as SystemInfoActiveModel,
		Model as SystemInfoModel,
	},
	system_resources::{
		ActiveModel as SystemResourcesActiveModel,
		Model as SystemResourcesModel,
	},
	server_node::Model as ServerNodeModel,
};
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};
use std::error::Error;

use super::resources::controller::SystemResourcesController;
use super::server_info::{
	ServerInfo,
	controller::ServerInfoController,
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
/// 
/// TODO: Make the rest of the models optional and fetch on creation or on-demand
pub struct ServerNodeController {
	pub db: DatabaseConnection,
	pub server_node: Option<ServerNode>,
	server_node_active_model: Option<ServerNodeActiveModel>,
	// Models
	pub server_location: Option<ServerLocationActiveModel>,
	pub system_resources: Option<SystemResourcesActiveModel>,
	pub system_info: Option<SystemInfoActiveModel>,
}

/// Constructors
/// 
/// 
impl ServerNodeController {
	/// Create new
	///
	/// Fetch resources on creation
	pub fn new(
		db: DatabaseConnection,
		server_node: Option<ServerNode>,
		server_node_active_model: Option<ServerNodeActiveModel>,
		server_location: Option<ServerLocationActiveModel>,
		system_resources: Option<SystemResourcesActiveModel>,
		system_info: Option<SystemInfoActiveModel>,
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
}

/// Local methods
/// 
/// Methods that don't act on the database
impl ServerNodeController {
	/// Get server node
	/// 
	/// The server node is cloned
	/// If server node doesn't exists create it
	pub fn get_server_node(&mut self) -> Result<ServerNode, Box<dyn Error>> {
		let server_node = match self.server_node.clone() {
			Some(server_node) => server_node,
			None => {
				let server_node = ServerNode::new(1)?;
				
				self.server_node = Some(server_node.clone());
				
				server_node
			}
		};
		
		Ok(server_node)
	}
	
	/// Get server location
	/// 
	/// 
	pub fn get_server_location(&self) -> Result<&ServerLocationActiveModel, Box<dyn Error>> {
		match &self.server_location {
			Some(location) => Ok(location),
			None => Err("Server location is not set".into()),
		}
	}
	
	/// Get system resources
	/// 
	/// 
	pub fn get_system_resources(&self) -> Result<&SystemResourcesActiveModel, Box<dyn Error>> {
		match &self.system_resources {
			Some(resources) => Ok(resources),
			None => Err("System resources are not set".into()),
		}
	}
	
	/// Get system info
	/// 
	/// 
	pub fn get_system_info(&self) -> Result<&SystemInfoActiveModel, Box<dyn Error>> {
		match &self.system_info {
			Some(info) => Ok(info),
			None => Err("System info is not set".into()),
		}
	}
	
	/// Get server node active model
	/// 
	/// 
	pub fn get_server_node_active_model(&self) -> Result<&ServerNodeActiveModel, Box<dyn Error>> {
		match &self.server_node_active_model {
			Some(model) => Ok(model),
			None => Err("Server node active model is not set".into()),
		}
	}
}

/// Utility methods
/// 
/// 
impl ServerNodeController {
	/// Get or create server location
	/// 
	/// 
	pub async fn get_or_create_server_location(&mut self) -> Result<ServerLocationActiveModel, Box<dyn Error>> {
		let server_location = match self.get_server_location() {
			Ok(location) => location.clone(),
            Err(_) => {
				// Get server node
				let server_node = self.get_server_node()?;
				
				// Create server info controller
                let server_info_controller = ServerInfoController::new(
                    self.db.clone(),
					server_node.location.clone()
                ).await?;
                
				let model = server_info_controller.insert()
					.await?
					.clone()
					.into_active_model();
				
				self.server_location = Some(model.clone());
				
				model
            }
		};
		
		Ok(server_location)
	}
	
	/// Get or create system resources
	/// 
	/// 
	pub async fn get_or_create_system_resources(&mut self) -> Result<SystemResourcesActiveModel, Box<dyn Error>> {
		let system_resources = match self.get_system_resources() {
			Ok(system_resources) => system_resources.clone(),
            Err(_) => {
				// Get server node
				let server_node = self.get_server_node()?;
				
				// Create system resources controller
                let mut system_resources_controller = SystemResourcesController::new(
                    self.db.clone(),
					Some(server_node.resources.clone())
                );
                
				system_resources_controller
					.insert_data()
					.await?;
				
				let model = system_resources_controller.get_resources_active_model()?;
				
				self.system_resources = Some(model.clone());
				
				model
            }
		};
		
		Ok(system_resources)
	}
	
	/// Get or create system info
	/// 
	/// 
	pub async fn get_or_create_system_info(&mut self) -> Result<SystemInfoActiveModel, Box<dyn Error>> {
		let system_info = match self.get_system_info() {
            Ok(info) => info.clone(),
            Err(_) => {
                // Get server node
                let server_node = self.get_server_node()?;
                
                // Create system info controller
                let system_info_controller = SystemInfoController::new(
                    self.db.clone(),
                    server_node.system_info.clone()
                ).await?;
                
                let model = system_info_controller.insert()
                    .await?
                    .clone()
                    .into_active_model();
                
                self.system_info = Some(model.clone());
                
                model
            }
        };
        
        Ok(system_info)
	}
	
	/// Get or insert server node active model
	///
	/// On creation the server node will be inserted, to make things faster
	/// 
	/// Every model is created and inserted in the database
	pub async fn get_or_create_server_node_active_model(
		&mut self,
	) -> Result<ServerNodeActiveModel, Box<dyn Error>> {
		let model = match self.get_server_node_active_model() {
			Ok(model) => model.clone(),
			Err(_) => {
				// Get or create server location
				let server_location_id = match self
					.get_or_create_server_location()
					.await?
					.id
					.clone()
					.take() {
					Some(id) => id,
					None => return Err("Server location id is not provided".into()),
				};
				
				// Get or create system resources
				let system_resource_id = match self
					.get_or_create_system_resources()
					.await?
					.id
					.clone()
					.take() {
					Some(id) => id,
					None => return Err("System resource id is not provided".into()),
				};
				
				// Get or create system info
				let system_info_id = match self
					.get_or_create_system_info()
					.await?
					.id
					.clone()
					.take() {
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
		self.get_or_create_server_node_active_model().await?;
		Ok(self)
	}
	
	/// Insert server node
	/// 
	/// 
	pub async fn insert_server_node(&mut self, server_node: ServerNode) -> Result<&mut Self, Box<dyn Error>> {
		self.server_node = Some(server_node);
		self.get_or_create_server_node_active_model().await?;
        
        Ok(self)
	}
	
	/// Delete
	///
	///
	pub async fn delete(&mut self) -> Result<&mut Self, Box<dyn Error>> {
		let server_node_active_model = self.get_or_create_server_node_active_model().await?;
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

/// Anonymous functions
/// 
/// Mostly for creating structures
impl ServerNodeController {
	/// Fetch server node side models
	/// 
	/// Temporal function for easier understanding
	pub async fn server_node_side_models(db: &DatabaseConnection, server_node_model: ServerNodeModel) -> Result<
		(ServerLocationModel, SystemInfoModel, SystemResourcesModel),
		Box<dyn Error>
	> {
		// Find server location
		let server_location_model = ServerInfoController::find_by_server_node_model(
			db.clone(),
			server_node_model.clone()
		).await?;
		
		// Find system resources
		let system_resources_model = SystemResourcesController::find_by_server_node_model(
			db.clone(),
			server_node_model.clone()
		).await?;
		
		// Find system info
		let system_info_model = SystemInfoController::find_by_server_node_model(
			db.clone(),
			server_node_model
		).await?;
		
		Ok((server_location_model, system_info_model, system_resources_model))
	}
	
	/// Create server node from id
	/// 
	/// 
	pub async fn server_node_from_id(
		db: DatabaseConnection,
		id: u32
	) -> Result<ServerNode, Box<dyn Error>> {
		// Find server node id
		let server_node_model = ServerNodeEntity::find_by_id(id).one(&db).await?;
		let server_node = match server_node_model {
			Some(server_node_model) => {
				// Take status
				let status = server_node_model.status.clone();
				let server_node_id = server_node_model.id.clone();
				
				// Find server location
				let server_location_model = ServerInfoController::find_by_server_node_model(db.clone(), server_node_model.clone())
					.await?;
				let server_location = match ServerInfo::from_model(server_location_model.clone()) {
					Some(server_location) => server_location,
					None => return Err("Couldn't convert server location model to server info".into()),
				};
				
				// Find system resources
				let system_resources_model = SystemResourcesController::find_by_server_node_model(db.clone(), server_node_model.clone())
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
				let system_info_model = SystemInfoController::find_by_server_node_model(db.clone(), server_node_model)
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
	
	/// Create new from server node id
	/// 
	/// FIXME: There are many redundant database roundtrips
	pub async fn new_from_server_node_id(
		db: DatabaseConnection,
		id: u32
	) -> Result<Self, Box<dyn Error>> {
		// FIXME: Server node side models are fetch first so there should be a way to pass them down to a new function as currently we are fetching the same data twice
		// FIXME: Server node model is fetch twice too
		
		// Find server node id
		let server_node_model = ServerNodeEntity::find_by_id(id).one(&db).await?;
		let server_node_model = match server_node_model {
			Some(server_node_model) => server_node_model,
			None => return Err("Server node not found".into()),
		};
		
		let (
			server_location_model,
			system_info_model,
			system_resources_model
		) = Self::server_node_side_models(&db, server_node_model)
			.await?;
		let server_node = ServerNodeController::server_node_from_id(db.clone(), id)
			.await?;
		
		let server_location_id = server_location_model.id.clone();
		let system_resources_id = system_resources_model.id.clone();
		let system_info_id = system_info_model.id.clone();
		
		Ok(Self {
			db,
			server_node: Some(server_node.clone()),
			server_node_active_model: Some(server_node.try_into_active_model(server_location_id, system_resources_id, system_info_id)?),
			server_location: Some(server_location_model.into_active_model()),
			system_resources: Some(system_resources_model.into_active_model()),
			system_info: Some(system_info_model.into_active_model()),
		})
	}
}

// #[cfg(test)]
// mod tests {
// 	use sea_orm::{DatabaseConnection, EntityTrait};
// 	use entity::server_node::Entity as ServerNodeEntity;
	
// 	use super::*;
// 	use crate::database::mysql_connection;
	
// 	#[test]
// 	async fn test_insert() {
// 		// Create a test database connection
// 		let db = mysql_connection()
// 			.await
// 			.unwrap();
		
// 		let server_node = ServerNode::new()
// 			.unwrap();
// 	}
// }
