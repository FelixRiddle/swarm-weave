//! Memory controller
//!
//! Handle system memory model
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::error::Error;
use entity::system_memory::{
	self,
	Entity as SystemMemoryEntity,
	Model as SystemMemoryModel,
};

/// Memory controller
///
///
pub struct MemoryController {
	pub db: DatabaseConnection,
}

impl MemoryController {
	pub fn new(db: DatabaseConnection) -> Self {
		Self { db }
	}
	
	/// Fetch system memory
	/// 
	/// 
	pub async fn get_system_memory_by_resources_id(&self, resource_id: i64) -> Result<SystemMemoryModel, Box<dyn Error>> {
		let memory_model = SystemMemoryEntity::find()
			.filter(system_memory::Column::SystemResourceId.eq(resource_id))
			.one(&self.db)
			.await?;
		
		match memory_model {
			Some(model) => Ok(model),
            None => Err("System memory not found".into())
		}
	}
}
