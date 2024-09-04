use sea_orm::{
	ActiveValue,
	DatabaseConnection,
	EntityTrait,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use entity::server_node::{
	Entity as ServerNodeEntity,
	ActiveModel as ServerNodeActiveModel,
};

pub mod server_info;
pub mod system_info;
pub mod resources;
pub mod storage;

pub use resources::Resources;
pub use system_info::SystemInfo;
pub use server_info::ServerInfo;

use crate::database::mysql_connection;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize, Serialize)]
pub enum ServerStatus {
    Online,
    Offline,
    Maintenance,
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
	
	// /// Try into active model
	// /// 
	// /// 
    // pub fn try_into_active_model(self) -> Result<ServerNodeActiveModel, Box<dyn Error>> {
    //     Ok(ServerNodeActiveModel {
    //         id: ActiveValue::Set(self.id as i32),
    //         location: ActiveValue::Set(self.location.into_active_model()),
    //         status: ActiveValue::Set(match self.status {
    //             ServerStatus::Online => "Online",
    //             ServerStatus::Offline => "Offline",
    //             ServerStatus::Maintenance => "Maintenance",
    //         }),
    //         resources: ActiveValue::Set(self.resources.into_active_model()),
    //         system_info: ActiveValue::Set(self.system_info.into_active_model()),
	// 		..Default::default()
    //     })
    // }
}

/// Server node controller
/// 
/// Mainly for database manipulation
pub struct ServerNodeController {
	pub db: DatabaseConnection,
	pub server_node: ServerNode,
}

impl ServerNodeController {
	/// Create new
	/// 
	/// Fetch resources locally
	pub async fn new() -> Result<Self, Box<dyn Error>> {
		let db = mysql_connection().await?;
		// On insert, id is ignored
		let server_node = ServerNode::new(1)?;
		
		Ok(Self { db, server_node })
	}
    
    // /// Insert
    // /// 
    // /// 
    // pub async fn insert(self) -> Result<Self, Box<dyn Error>> {
    //     let insert_result = self.server_node.clone().into_active_model().insert(&self.db).await?;
    //     assert_eq!(insert_result.rows_affected, 1);
        
    //     Ok(self)
    // }
    
    // /// Update
    // /// 
    // /// 
    // pub async fn update(self) -> Result<Self, Box<dyn Error>> {
    //     let update_result = self.server_node.clone().into_active_model().update(&self.db).await?;
    //     assert_eq!(update_result.rows_affected, 1);
        
    //     Ok(self)
    // }
    
    // /// Find
    // /// 
    // /// 
    // pub async fn find(self, id: u32) -> Result<Self, Box<dyn Error>> {
    //     let found_server_node = ServerNodeEntity::find_by_id(id).one(&self.db).await?;
    //     match found_server_node {
    //         Some(server_node) => {
    //             let server_node = server_node.try_into_model()?;
    //             Ok(Self { db: self.db, server_node })
    //         },
    //         None => Err("Server node not found".into()),
    //     }
    // }
    
    /// Delete
    /// 
    /// 
    pub async fn delete(self, id: u32) -> Result<Self, Box<dyn Error>> {
        let delete_result = ServerNodeEntity::delete_by_id(id).exec(&self.db).await?;
        assert_eq!(delete_result.rows_affected, 1);
        
        Ok(self)
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
}
