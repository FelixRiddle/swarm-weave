use entity::{
    system_core::{
		Entity as SystemCoreEntity,
        ActiveModel as SystemCoreActiveModel
	},
    system_resources::ActiveModel as SystemResourcesActiveModel,
};
use sea_orm::{
	ModelTrait,
    ActiveValue,
    DatabaseConnection, TryIntoModel,
};
use serde::{Deserialize, Serialize};
use std::error::Error;

use super::{
	to_f32,
	Resources
};

/// TODO: Rename to CpuCore or Core
/// 
/// 
#[derive(Clone, Deserialize, Serialize)]
pub struct SystemCore {
    pub usage_percentage: f64,
    pub free_percentage: f64,
}

/// System core controller
/// 
/// Because I'm tired of using references
pub struct SystemCoreController {
	pub db: DatabaseConnection,
	pub system_resources: Resources,
	pub system_resources_instance: SystemResourcesActiveModel,
}

/// Database and Resources
/// 
/// 
impl SystemCoreController {
	/// Create new
	/// 
	/// 
	pub fn new(
		db: DatabaseConnection,
        system_resources: Resources,
        system_resources_instance: SystemResourcesActiveModel,
	) -> Self {
		Self {
			db,
			system_resources,
			system_resources_instance,
		}
	}
	
	/// Get id or throw error
	/// 
	/// 
	pub fn id(&self) -> Result<i64, Box<dyn Error>> {
		let mut system_resources_id = self.system_resources_instance.id.clone();
		let system_resources_id = match system_resources_id.take() {
            Some(id) => id,
            None => return Err(format!("Failed to create system core instance with system resources id").into()),
        };
		
		Ok(system_resources_id)
	}
	
	/// Create system core instance
	/// 
	/// 
	pub fn create_system_core_instance(
		&self,
		cpu: &SystemCore,
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
		let system_core_instances: Result<Vec<SystemCoreActiveModel>, Box<dyn Error>> = self.system_resources
            .cpus
            .iter()
            .map(|cpu| self.create_system_core_instance(cpu))
            .collect();
		
		system_core_instances
	}
	
	/// Update cores from resources
	/// 
	/// 
	pub async fn update_all_cores(
		&self,
	) -> Result<(), Box<dyn Error>> {
        // Update system cores
        let system_core_instances = self.create_cores()?;
		
		// Cpus don't have identification
		// Find related cpus
		let cpus = self.system_resources_instance
			.clone()
			.try_into_model()?
			.find_related(SystemCoreEntity)
			.all(&self.db)
			.await?;
		
		// Remove difference
		let diff = i32::try_from(cpus.len())? - i32::try_from(system_core_instances.len())?;
		println!("Current instances: {}", system_core_instances.len());
		println!("Existing instances: {}", cpus.len());
		println!("Absolute difference: {}", diff);
		
		// It's done like this because cores cannot be identified
		if diff > 0 {
			// TODO: We have to remove some, and update those that remain
			// Remove extras
			let mut current: usize = 0;
			while current < usize::try_from(diff)? {
				cpus[current]
					.clone()
					.delete(&self.db)
					.await?;
				
				current += 1;
			}
		} else {
			// It's negative so there are less in the database
			// let diff = diff * -1;
			
			// Still updating this is a pain
			
			// This is just a reference
			// for system_core_instance in system_core_instances {
			// 	system_core_instance.save(db).await?;
			// }
		}
		
		Ok(())
	}
}
