use entity::{
	system_core::{
		self,
		ActiveModel as SystemCoreActiveModel,
		Entity as SystemCoreEntity,
		Model as SystemCoreModel,
	},
	system_resources::ActiveModel as SystemResourcesActiveModel,
};
use sea_orm::{
	ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder
};
use std::error::Error;

use super::{to_f32, Resources, CpuCore};

/// System core controller
///
/// Because I'm tired of using references
pub struct CpuCoreController {
	pub db: DatabaseConnection,
	system_resources: Option<Resources>,
	system_resources_instance: Option<SystemResourcesActiveModel>,
}

/// Database and Resources
///
///
impl CpuCoreController {
	/// Create new
	///
	///
	pub fn new(
		db: DatabaseConnection,
		system_resources: Option<Resources>,
		system_resources_instance: Option<SystemResourcesActiveModel>,
	) -> Self {
		Self {
			db,
			system_resources,
			system_resources_instance,
		}
	}
	
	/// Set system resources instance
	/// 
	/// 
	pub fn set_system_resources_instance(&mut self, system_resources_instance: SystemResourcesActiveModel) {
        self.system_resources_instance = Some(system_resources_instance);
    }
	
	/// Get resources
	/// 
	/// 
	pub fn get_resources(&self) -> Result<Resources, Box<dyn Error>> {
		let resources = match self.system_resources.clone() {
			Some(resources) => resources,
			None => Resources::fetch_resources()?
		};
		
		Ok(resources)
	}
	
	/// Get system resources instance
	/// 
	/// 
	pub fn get_system_resources_instance(&self) -> Result<SystemResourcesActiveModel, Box<dyn Error>> {
		let system_resources_instance = match self.system_resources_instance.clone() {
			Some(instance) => instance,
			None => return Err(format!("System resources instance doesn't exists, please fetch it or create it").into())
		};
		
		Ok(system_resources_instance)
    }
	
	/// Get system resources instance id
	/// 
	/// In case it's not found throws an error
	pub fn id(&self) -> Result<i64, Box<dyn Error>> {
		let system_resources_instance = self.get_system_resources_instance()?;
		
		let system_resources_id = match system_resources_instance.id.clone().take() {
			Some(id) => id,
			None => {
				return Err(format!(
					"Failed to create system core instance with system resources id"
				)
				.into())
			}
		};

		Ok(system_resources_id)
	}
	
	/// Create system core instance
	///
	///
	pub fn create_system_core_instance(
		&self,
		cpu: &CpuCore,
	) -> Result<SystemCoreActiveModel, Box<dyn Error>> {
		let system_resources_id = self.id()?;

		// Create system core
		let system_core_instance = SystemCoreActiveModel {
			usage_percentage: ActiveValue::Set(to_f32(cpu.usage_percentage)?),
			free_percentage: ActiveValue::Set(to_f32(cpu.free_percentage)?),
			system_resource_id: ActiveValue::Set(Some(system_resources_id)),
			..Default::default() // all other attributes are `NotSet`
		};

		Ok(system_core_instance)
	}
	
	/// Create cores from resources
	///
	///
	pub fn create_cores(&self) -> Result<Vec<SystemCoreActiveModel>, Box<dyn Error>> {
		let system_resources = self.get_resources()?;
		let system_core_instances: Result<Vec<SystemCoreActiveModel>, Box<dyn Error>> = system_resources
			.cpus
			.iter()
			.map(|cpu| self.create_system_core_instance(cpu))
			.collect();
		
		system_core_instances
	}
	
	// /// Insert cores
	// /// 
	// /// TODO: Insert cores
	// pub async fn insert_cores(&self) -> Result<Vec<SystemCoreActiveModel>, Box<dyn Error>> {
		
	// 	let system_resources_instance = self.get_system_resources_instance()?;
	// 	let system_resources = self.get_resources()?;
		
	// 	// Cpus don't have identification
	// 	// Find related cpus
	// 	let mut cpus: Vec<SystemCoreModel> = system_resources_instance
	// 		.clone()
	// 		.try_into_model()?
	// 		.find_related(SystemCoreEntity)
	// 		.all(&self.db)
	// 		.await?;
		
	// 	let mut cores = Vec::new();
		
	// 	Ok(cores)
	// }
	
	// /// Update cores unchangeable
	// /// 
	// /// This function assumes that you don't change the processor ever
	// /// 
	// /// TODO: Update cores
	// pub async fn update_cores(&self) -> Result<Vec<SystemCoreActiveModel>, Box<dyn Error>> {
		
	// 	let system_resources_instance = self.get_system_resources_instance()?;
	// 	let system_resources = self.get_resources()?;
		
	// 	// Cpus don't have identification
	// 	// Find related cpus
	// 	let mut cpus: Vec<SystemCoreModel> = system_resources_instance
	// 		.clone()
	// 		.try_into_model()?
	// 		.find_related(SystemCoreEntity)
	// 		.all(&self.db)
	// 		.await?;
		
	// 	let mut cores = Vec::new();
		
	// 	Ok(cores)
	// }
	
	/// Find cores
	/// 
	/// Make sure you set system resources instance
	/// 
	/// Returns models
	pub async fn find_cores(&self) -> Result<Vec<SystemCoreModel>, Box<dyn Error>> {
		let system_resources_id = self.id()?;
		
		let cpu_core_models = SystemCoreEntity::find()
			.filter(system_core::Column::SystemResourceId.eq(system_resources_id))
			.order_by_asc(system_core::Column::Id)
			.all(&self.db)
			.await?;
		
		Ok(cpu_core_models)
	}
	
	/// Find cores 
	/// 
	/// By using system resources id
	pub async fn find_cores_by_resources_id(&self, system_resources_id: i64) -> Result<Vec<SystemCoreModel>, Box<dyn Error>> {
		let cpu_core_models = SystemCoreEntity::find()
			.filter(system_core::Column::SystemResourceId.eq(system_resources_id))
			.order_by_asc(system_core::Column::Id)
			.all(&self.db)
			.await?;
		
		Ok(cpu_core_models)
	}
}
