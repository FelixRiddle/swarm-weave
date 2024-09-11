//! Storage controller
//! 
//! 
use entity::storage_device::{
	self,
	Entity as StorageDeviceEntity,
	Model as StorageDeviceModel,
};
use sea_orm::{
	ColumnTrait,
	DatabaseConnection,
	EntityTrait,
	QueryFilter,
};
use std::error::Error;

pub struct StorageController {
	pub db: DatabaseConnection,
}

impl StorageController {
	pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
	
	/// Fetch system storage
	/// 
	/// 
	pub async fn find_by_resources_id(&self, resource_id: i64) -> Result<Vec<StorageDeviceModel>, Box<dyn Error>> {
		let devices = StorageDeviceEntity::find()
			.filter(storage_device::Column::SystemResourceId.eq(resource_id))
			.all(&self.db)
			.await?;
		
		Ok(devices)
	}
}
