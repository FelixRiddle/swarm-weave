//! Node
//! 
//! Represents a server node
use entity::server_node::ActiveModel as ServerNodeActiveModel;
use entity::sea_orm_active_enums::Status;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use std::error::Error;
use strum_macros::Display;

pub mod controller;
pub mod resources;
pub mod server_info;
pub mod system_info;

use resources::Resources;
use server_info::ServerInfo;
use system_info::SystemInfo;

#[derive(Clone, Debug, Display, PartialEq, Deserialize, Serialize)]
pub enum ServerStatus {
	Online,
	Offline,
	Maintenance,
}

impl ServerStatus {
	/// Convert from status
	/// 
	/// I don't know why but this operation doesn't works with the trait, neither with the '.into()' function
	pub fn from_status(status: Status) -> Self {
		match status {
            Status::Online => ServerStatus::Online,
            Status::Offline => ServerStatus::Offline,
            Status::Maintenance => ServerStatus::Maintenance,
        }
	}
}

/// From server status to status
/// 
/// 
impl From<ServerStatus> for Status {
	fn from(status: ServerStatus) -> Self {
		match status {
			// TODO: Should add 'Untracked' to signify that the status is not being tracked
			ServerStatus::Online => Status::Online,
			ServerStatus::Offline => Status::Offline,
			ServerStatus::Maintenance => Status::Maintenance,
		}
	}
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ServerNode {
	pub location: ServerInfo,
	pub status: ServerStatus,
	pub resources: Resources,
	pub system_info: SystemInfo,
}

impl ServerNode {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		Ok(Self {
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
			status: ActiveValue::Set(Some(self.status.into())),
			server_location_id: ActiveValue::Set(Some(server_location_id)),
			system_resource_id: ActiveValue::Set(Some(resource_id)),
			system_info_id: ActiveValue::Set(Some(system_info_id)),
			..Default::default()
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_server_node_new() {
		let server_node = ServerNode::new().unwrap();
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
