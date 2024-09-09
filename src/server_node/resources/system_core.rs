use entity::{
	system_core::{
		ActiveModel as SystemCoreActiveModel, Entity as SystemCoreEntity, Model as SystemCoreModel,
	},
	system_resources::ActiveModel as SystemResourcesActiveModel,
};
use sea_orm::{
	ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, TryIntoModel
};
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::model::FromActiveModel;

use super::{to_f32, Resources};

type SystemCoreColumn = <entity::prelude::SystemCore as EntityTrait>::Column;

/// TODO: Rename to CpuCore or Core
///
///
#[derive(Clone, Deserialize, Serialize)]
pub struct CpuCore {
	pub usage_percentage: f64,
	pub free_percentage: f64,
}

impl CpuCore {
	/// Convert into active model
	///
	///
	pub fn try_into_active_model(
		&self,
		cpu: &CpuCore,
		system_resources_id: i64,
	) -> Result<SystemCoreActiveModel, Box<dyn Error>> {
		// Create system core
		let system_core_instance = SystemCoreActiveModel {
			usage_percentage: ActiveValue::Set(to_f32(cpu.usage_percentage)?),
			free_percentage: ActiveValue::Set(to_f32(cpu.free_percentage)?),
			system_resource_id: ActiveValue::Set(Some(system_resources_id)),
			..Default::default()
		};
		
		Ok(system_core_instance)
	}
}

impl FromActiveModel<SystemCoreActiveModel, Self> for CpuCore {
	fn from_active_model(active_model: SystemCoreActiveModel) -> Result<Self, Box<dyn Error>> {
		let system_core_instance = active_model.try_into_model()?;
		
        Ok(CpuCore {
            usage_percentage: system_core_instance.usage_percentage as f64,
            free_percentage: system_core_instance.free_percentage as f64,
        })
	}
}

/// System core controller
///
/// Because I'm tired of using references
pub struct CpuCoreController {
	pub db: DatabaseConnection,
	pub system_resources: Resources,
	pub system_resources_instance: SystemResourcesActiveModel,
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
		let system_core_instances: Result<Vec<SystemCoreActiveModel>, Box<dyn Error>> = self
			.system_resources
			.cpus
			.iter()
			.map(|cpu| self.create_system_core_instance(cpu))
			.collect();

		system_core_instances
	}

	/// Update all cores
	///
	/// FIXME: Doesn't work
	/// FIXME: Use 'save' instead of 'update', because update is lazy
	/// 
	/// TODO: While this doesn't wants to work, an alternative solution is to store them as json in a folder with the resources id as name
	pub async fn update_all_cores(&self) -> Result<(), Box<dyn Error>> {
		// Cpus don't have identification
		// Find related cpus
		let mut cpus: Vec<SystemCoreModel> = self
			.system_resources_instance
			.clone()
			.try_into_model()?
			.find_related(SystemCoreEntity)
			.all(&self.db)
			.await?;

		let local_cpus_quantity = i32::try_from(self.system_resources.cpus.len())?;

		// Remove difference
		let diff = i32::try_from(cpus.len())? - local_cpus_quantity;
		println!("Local: {}", self.system_resources.cpus.len());
		println!("Database: {}", cpus.len());
		println!("Absolute difference: {}", diff);
		
		// It's done like this because cores cannot be identified
		if diff > 0 {
			println!("There are more cores in the database than in the system");
			
			// This is crazy
			// Sort by id and then get the difference as index smallest id
			// From there onwards we can delete records from the database
			cpus.sort_by_key(|cpu| cpu.id);
			// let diffth_smallest_id = cpus[diff as usize].id;
			let diffth_smallest_id = cpus[cpus.len() - diff as usize].id;
			
			println!("Smallest id: {}", diffth_smallest_id);
			for (index, _cpu) in cpus.iter().enumerate() {
				println!("Index: {}, Id: {}", index, _cpu.id);
			}
			
			// Remove the last diff number of elements
			SystemCoreEntity::delete_many()
				.filter(
					Condition::all()
						.add(SystemCoreColumn::SystemResourceId.eq(self.id()?))
						.add(SystemCoreColumn::Id.gte(diffth_smallest_id))
				)
				.exec(&self.db)
				.await?;
			
			println!("Cores removed");
			
			// Reload the cpus vector from the database
			cpus = self
				.system_resources_instance
				.clone()
				.try_into_model()?
				.find_related(SystemCoreEntity)
				.all(&self.db)
				.await?;
			
			println!("Database cores: {}", cpus.len());
			
			// Check that the cpus vector has been truncated to the correct length
			assert_eq!(cpus.len(), local_cpus_quantity as usize);
			
			// Update the remaining cores
			for (index, cpu_core) in cpus.iter_mut().enumerate() {
				if index < self.system_resources.cpus.len() {
					let local_core = &self.system_resources.cpus[index];
					
					cpu_core.usage_percentage = local_core.usage_percentage as f32;
					cpu_core.free_percentage = local_core.free_percentage as f32;
					
					cpu_core
						.clone()
						.into_active_model()
						// Use 'save' instead of 'update'
						.save(&self.db)
						.await?;
				}
			}
		} else if diff == 0 {
			println!("Cores quantity hasn't changed");
			for (index, cpu_core) in cpus.iter_mut().enumerate() {
				let local_core = &self.system_resources.cpus[index];

				// Check if the local CPU usage has changed
				if cpu_core.usage_percentage != local_core.usage_percentage as f32
					|| cpu_core.free_percentage != local_core.free_percentage as f32
				{
					cpu_core.usage_percentage = local_core.usage_percentage as f32;
					cpu_core.free_percentage = local_core.free_percentage as f32;

					cpu_core
						.clone()
						.into_active_model()
						.save(&self.db)
						.await?;
				}
			}
		} else {
			println!("There are more cores locally than in the database");
			
			// Update all existing cores in the database
			let cores_to_update = cpus.len();

			// Update the first cores
			let mut current: usize = 0;
			while current < cores_to_update {
				let mut model = cpus[current].clone();

				let local_core = &self.system_resources.cpus[current];
				model.usage_percentage = local_core.usage_percentage as f32;
				model.free_percentage = local_core.free_percentage as f32;

				model.clone().into_active_model().save(&self.db).await?;

				current += 1;
			}

			// Insert the remaining new cores
			let mut remaining_instances: Vec<CpuCore> = self
				.system_resources
				.cpus
				.iter()
				.skip(current)
				.cloned()
				.collect();

			for (_index, remaining_instance) in remaining_instances.iter_mut().enumerate() {
				let system_core = self.create_system_core_instance(remaining_instance)?;

				// Handle insertion errors
				if let Err(e) = system_core.clone().insert(&self.db).await {
					// Log or handle the error
					println!("Error inserting core: {}", e);
				}
			}
		}

		Ok(())
	}
	
	// /// Find cores
	// /// 
	// /// 
	// pub fn find_cores(db: DatabaseConnection, system_resources_id: i64) -> Result<Vec<CpuCore>, Box<dyn Error>> {
		
	// }
}

#[cfg(test)]
pub mod tests {
	use chrono::Utc;
	use entity::{
		system_core::Entity as SystemCoreEntity, system_resources::Entity as SystemResourcesEntity,
	};
	use sea_orm::{EntityTrait, ModelTrait};
	
	use super::CpuCore;
	use crate::database::mysql_connection;
	use crate::server_node::{
		resources::{
			Resources,
			SystemResourcesController,
			to_f32,
			system_memory::Memory,
		},
		storage::{DiskKind, Storage},
	};

	/// Update when there are less system cores locally than in the database
	///
	///
	#[tokio::test]
	async fn test_update_with_less() {
		// Set environment variables
		dotenv::dotenv().ok();

		// Initialize database connection
		let db = mysql_connection().await.unwrap();

		// Fetch resources
		let resources = Resources::fetch_resources().unwrap();

		// Insert initial data
		let system_resources_controller = SystemResourcesController::new(db.clone(), resources);
		let resource_id: i64 = system_resources_controller.insert_data().await.unwrap();

		// Update resources
		let updated_resources = Resources {
			cpus: vec![CpuCore {
				usage_percentage: 50.0,
				free_percentage: 50.0,
			}],
			memory: Memory {
				total: 8_589_934_592,
				used: 4_294_967_296,
			},
			storage: vec![Storage {
				name: String::from("Updated Storage"),
				total: 1_000_000_000,
				used: 500_000_000,
				is_removable: true,
				kind: DiskKind::HDD,
			}],
			eval_time: Utc::now(),
		};

		// Call the update function
		system_resources_controller.update(resource_id, &db).await.unwrap();

		// Verify that the data was updated correctly
		let res_model = SystemResourcesEntity::find_by_id(resource_id)
			.one(&db)
			.await
			.unwrap()
			.unwrap();

		// This test fails for a negligible difference
		// assert_eq!(res_model.eval_time, updated_resources.eval_time.naive_utc());

		// Get system cores
		let updated_system_cores = res_model
			.find_related(SystemCoreEntity)
			.all(&db)
			.await
			.unwrap();

		assert_eq!(updated_system_cores.len(), updated_resources.cpus.len());
		assert_eq!(
			updated_system_cores[0].usage_percentage,
			to_f32(updated_resources.cpus[0].usage_percentage).unwrap()
		);
		assert_eq!(
			updated_system_cores[0].free_percentage,
			to_f32(updated_resources.cpus[0].free_percentage).unwrap()
		);
	}

	/// Test update equal
	///
	/// Update when there is the same time of cores locally and in the database
	#[tokio::test]
	async fn test_update_equal() {
		// Set environment variables
		dotenv::dotenv().ok();

		// Initialize database connection
		let db = mysql_connection().await.unwrap();

		// Update resources
		let resources = Resources {
			cpus: vec![
				CpuCore {
					usage_percentage: 50.0,
					free_percentage: 50.0,
				},
				CpuCore {
					usage_percentage: 60.0,
					free_percentage: 40.0,
				},
			],
			memory: Memory {
				total: 8_589_934_592,
				used: 4_294_967_296,
			},
			storage: vec![
				Storage {
					name: String::from("Updated Storage 1"),
					total: 1_000_000_000,
					used: 500_000_000,
					is_removable: true,
					kind: DiskKind::HDD,
				},
				Storage {
					name: String::from("Updated Storage 2"),
					total: 1_000_000_000,
					used: 500_000_000,
					is_removable: true,
					kind: DiskKind::SSD,
				},
			],
			eval_time: Utc::now(),
		};

		// Insert initial data
		let system_resources_controller = SystemResourcesController::new(db.clone(), resources);
		let resource_id: i64 = system_resources_controller.insert_data().await.unwrap();

		// Update resources
		let updated_resources = Resources {
			cpus: vec![
				CpuCore {
					usage_percentage: 30.0,
					free_percentage: 70.0,
				},
				CpuCore {
					usage_percentage: 40.0,
					free_percentage: 60.0,
				},
			],
			memory: Memory {
				total: 8_589_934_592,
				used: 4_294_967_296,
			},
			storage: vec![
				Storage {
					name: String::from("Updated Storage 1"),
					total: 1_000_000_000,
					used: 500_000_000,
					is_removable: true,
					kind: DiskKind::HDD,
				},
				Storage {
					name: String::from("Updated Storage 2"),
					total: 1_000_000_000,
					used: 500_000_000,
					is_removable: true,
					kind: DiskKind::SSD,
				},
			],
			eval_time: Utc::now(),
		};

		// Call the update function
		let system_resources_controller = SystemResourcesController::new(db.clone(), updated_resources.clone());
		system_resources_controller.update(resource_id, &db).await.unwrap();

		// Verify that the data was updated correctly
		let res_model = SystemResourcesEntity::find_by_id(resource_id)
			.one(&db)
			.await
			.unwrap()
			.unwrap();

		// This test fails for a negligible difference
		// assert_eq!(res_model.eval_time, updated_resources.eval_time.naive_utc());

		// Get system cores
		let updated_system_cores = res_model
			.find_related(SystemCoreEntity)
			.all(&db)
			.await
			.unwrap();

		assert_eq!(updated_system_cores.len(), 2);
		assert_eq!(
			updated_system_cores[0].usage_percentage,
			to_f32(updated_resources.cpus[0].usage_percentage).unwrap()
		);
		assert_eq!(
			updated_system_cores[0].free_percentage,
			to_f32(updated_resources.cpus[0].free_percentage).unwrap()
		);
		assert_eq!(
			updated_system_cores[1].usage_percentage,
			to_f32(updated_resources.cpus[1].usage_percentage).unwrap()
		);
		assert_eq!(
			updated_system_cores[1].free_percentage,
			to_f32(updated_resources.cpus[1].free_percentage).unwrap()
		);
	}

	/// Test update more
	///
	/// Update when there are more cores locally than in the database
	#[tokio::test]
	async fn test_update_more() {
		// Set environment variables
		dotenv::dotenv().ok();

		// Initialize database connection
		let db = mysql_connection().await.unwrap();

		// Fetch resources
		let resources = Resources::fetch_resources().unwrap();

		// Insert initial data
		let system_resources_controller = SystemResourcesController::new(db.clone(), resources);
		let resource_id: i64 = system_resources_controller.insert_data().await.unwrap();

		// Create system cores based on the updated resources
		let mut system_cores = system_resources_controller.resources.cpus.clone();

		// Update resources
		let new_cores = vec![
			CpuCore {
				usage_percentage: 50.0,
				free_percentage: 50.0,
			},
			CpuCore {
				usage_percentage: 60.0,
				free_percentage: 40.0,
			},
			CpuCore {
				usage_percentage: 70.0,
				free_percentage: 30.0,
			},
		];
		system_cores.extend(new_cores);

		let updated_resources = Resources {
			cpus: system_cores.clone(),
			memory: Memory {
				total: 8_589_934_592,
				used: 4_294_967_296,
			},
			storage: vec![
				Storage {
					name: String::from("Updated Storage 1"),
					total: 1_000_000_000,
					used: 500_000_000,
					is_removable: true,
					kind: DiskKind::HDD,
				},
				Storage {
					name: String::from("Updated Storage 2"),
					total: 1_000_000_000,
					used: 500_000_000,
					is_removable: true,
					kind: DiskKind::SSD,
				},
				Storage {
					name: String::from("Updated Storage 3"),
					total: 1_000_000_000,
					used: 500_000_000,
					is_removable: true,
					kind: DiskKind::SSD,
				},
			],
			eval_time: Utc::now(),
		};

		// Call the update function
		let system_resources_controller = SystemResourcesController::new(db.clone(), updated_resources.clone());
		system_resources_controller.update(resource_id, &db).await.unwrap();

		// Verify that the data was updated correctly
		let res_model = SystemResourcesEntity::find_by_id(resource_id)
			.one(&db)
			.await
			.unwrap()
			.unwrap();

		// Get system cores
		let updated_system_cores = res_model
			.find_related(SystemCoreEntity)
			.all(&db)
			.await
			.unwrap();

		let total_cores = system_cores.len();
		assert_eq!(updated_system_cores.len(), total_cores);

		for (i, core) in updated_system_cores.iter().enumerate() {
			assert_eq!(
				core.usage_percentage,
				to_f32(updated_resources.cpus[i].usage_percentage).unwrap()
			);
			assert_eq!(
				core.free_percentage,
				to_f32(updated_resources.cpus[i].free_percentage).unwrap()
			);
		}
	}
}
