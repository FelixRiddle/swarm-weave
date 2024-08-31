use entity::{
    system_core::{
		Entity as SystemCoreEntity,
        ActiveModel as SystemCoreActiveModel
	},
    system_resources::ActiveModel as SystemResources,
};
// use futures::{executor::ThreadPool, join};
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
#[derive(Deserialize, Serialize)]
pub struct SystemCore {
    pub usage_percentage: f64,
    pub free_percentage: f64,
}

/// Database and Resources
/// 
/// 
impl SystemCore {
	/// Create system core instance
	/// 
	/// 
	pub fn create_system_core_instance(
		cpu: &SystemCore,
		system_resources_id: i64,
	) -> Result<SystemCoreActiveModel, Box<dyn Error>> {
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
	pub fn create_cores(res: &Resources, system_resources_id: i64) -> Result<Vec<SystemCoreActiveModel>, Box<dyn Error>> {
		let system_core_instances: Result<Vec<SystemCoreActiveModel>, Box<dyn Error>> = res
            .cpus
            .iter()
            .map(|cpu| SystemCore::create_system_core_instance(cpu, system_resources_id))
            .collect();
		
		system_core_instances
	}
	
	/// Update cores from resources
	/// 
	/// 
	pub async fn update_cores(
		res: &Resources,
		system_resources_id: i64,
		db: &DatabaseConnection,
		system_resources_instance: SystemResources
	) -> Result<(), Box<dyn Error>> {
        // Update system cores
        let system_core_instances = SystemCore::create_cores(res, system_resources_id)?;
		
		// Cpus don't have identification
		// Find related cpus
		let cpus = system_resources_instance
			.clone()
			.try_into_model()?
			.find_related(SystemCoreEntity)
			.all(db)
			.await?;
		
		// Remove difference
		let diff = i32::try_from(cpus.len())? - i32::try_from(system_core_instances.len())?;
		println!("Current instances: {}", system_core_instances.len());
		println!("Existing instances: {}", cpus.len());
		println!("Absolute difference: {}", diff);
		// It's done like this because cores cannot be identified
		if diff > 0 {
			// TODO: We have to remove some, and update those that remain
			
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
	
	// /// Update cores v2
	// /// 
	// /// Generated with tabnine
	// pub fn update_cores_v2(
	// 	&self,
	// 	updated_resources: &Resources,
	// 	system_resources_id: i64,
	// 	db: &DatabaseConnection,
	// ) -> Result<(), anyhow::Error> {
	// 	// Get existing system cores
	// 	let existing_system_cores = SystemCore::find_by_system_resources_id(system_resources_id)
	// 		.all(db)
	// 		.await?
	// 		.unwrap();
	
	// 	// Create a map of existing cores for faster lookup
	// 	let existing_cores_map: HashMap<String, entity::system_core::Model> = existing_system_cores
	// 		.iter()
	// 		.map(|core| (core.name.clone(), core.clone()))
	// 		.collect();
	
	// 	// Iterate over the updated cores and update or insert them
	// 	for updated_core in &updated_resources.cpus {
	// 		if let Some(existing_core) = existing_cores_map.get(&updated_core.name) {
	// 			// Update existing core
	// 			existing_core.update(|m| {
	// 				m.usage_percentage = to_f32(updated_core.usage_percentage)?;
	// 				m.free_percentage = to_f32(updated_core.free_percentage)?;
	// 				Ok(())
	// 			})
	// 			.exec(db)
	// 			.await?;
	// 		} else {
	// 			// Insert new core
	// 			let new_core = entity::system_core::Model::new(updated_core.name.clone())
	// 				.set_system_resources_id(system_resources_id)
	// 				.set_usage_percentage(to_f32(updated_core.usage_percentage)?)
	// 				.set_free_percentage(to_f32(updated_core.free_percentage)?);
	
	// 			new_core.insert(db).await?;
	// 		}
	// 	}
	
	// 	// Delete any remaining cores that are not in the updated resources
	// 	let remaining_cores: Vec<entity::system_core::Model> = existing_system_cores
	// 		.into_iter()
	// 		.filter(|core| !updated_resources.cpus.iter().any(|updated_core| updated_core.name == core.name))
	// 		.collect();
	
	// 	for remaining_core in remaining_cores {
	// 		remaining_core.delete(db).await?;
	// 	}
	
	// 	Ok(())
	// }
}
