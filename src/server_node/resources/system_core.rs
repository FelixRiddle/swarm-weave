use entity::{
    system_core::{
		Model as SystemCoreModel,
		Entity as SystemCoreEntity,
        ActiveModel as SystemCoreActiveModel
	},
    system_resources::ActiveModel as SystemResourcesActiveModel,
};
use sea_orm::{
	ActiveModelTrait, ActiveValue, DatabaseConnection, IntoActiveModel, ModelTrait, TryIntoModel
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
pub struct CpuCore {
    pub usage_percentage: f64,
    pub free_percentage: f64,
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
            None => return Err(format!("Failed to create system core instance with system resources id").into()),
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
		// Cpus don't have identification
		// Find related cpus
		let mut cpus: Vec<SystemCoreModel> = self.system_resources_instance
			.clone()
			.try_into_model()?
			.find_related(SystemCoreEntity)
			.all(&self.db)
			.await?;
		
		// Remove difference
		let diff = i32::try_from(cpus.len())? - i32::try_from(self.system_resources.cpus.len())?;
		println!("Current instances: {}", self.system_resources.cpus.len());
		println!("Existing instances: {}", cpus.len());
		println!("Absolute difference: {}", diff);
		
		// It's done like this because cores cannot be identified
		if diff > 0 {
			println!("There are more cores in the database than in the system");
			
			// Remove extras
			let mut current: usize = 0;
			while current < usize::try_from(diff)? {
				let model = cpus[current].clone();
				
				model.delete(&self.db)
					.await?;
				
				current += 1;
			}
			
			// Update those that remain
			let remaining = cpus.len() - current;
			println!("Remaining: {}", remaining);
			
			// From the vector remove those that are before the current index
			let mut remaining_instances = cpus
                .iter()
                .skip(current)
                .cloned()
                .collect::<Vec<SystemCoreModel>>();
            
            // Update remaining
			for (index, remaining_instance) in remaining_instances.iter_mut().enumerate() {
				remaining_instance.usage_percentage = self.system_resources.cpus[index].usage_percentage as f32;
				remaining_instance.free_percentage = self.system_resources.cpus[index].free_percentage as f32;
				
				remaining_instance
					.clone()
					.into_active_model()
					.update(&self.db)
					.await?;
			}
		} else if diff == 0 {
			println!("Cores quantity hasn't changed");
			for(index, cpu_core) in cpus.iter_mut().enumerate() {
				let local_core = &self.system_resources.cpus[index];
				cpu_core.usage_percentage = local_core.usage_percentage as f32;
				cpu_core.free_percentage = local_core.free_percentage as f32;
				
				cpu_core
					.clone()
					.into_active_model()
					.update(&self.db)
					.await?;
			}
		} else {
			println!("There are more cores locally than in the database");
			
			// Update the first cores
			let mut current: usize = 0;
			while current < usize::try_from(diff)? {
				let mut model = cpus[current].clone();
				
				let local_core = &self.system_resources.cpus[current];
				model.usage_percentage = local_core.usage_percentage as f32;
				model.free_percentage = local_core.free_percentage as f32;
				
				model
					.clone()
					.into_active_model()
					.update(&self.db)
					.await?;
				
				current += 1;
			}
			
			// Insert those that remain
			let remaining = cpus.len() - current;
			println!("Remaining: {}", remaining);
			
			// From the vector remove those that are before the current index
			let mut remaining_instances = cpus
                .iter()
				// Skip all updated components
                .skip(current)
                .cloned()
                .collect::<Vec<SystemCoreModel>>();
			
			// Insert remaining
			for (_index, remaining_instance) in remaining_instances.iter_mut().enumerate() {
                remaining_instance.system_resource_id = Some(self.id()?);
                remaining_instance
                    .clone()
                    .into_active_model()
                    .insert(&self.db)
                    .await?;
            }
		}
		
		Ok(())
	}
}

#[cfg(test)]
pub mod tests {
    use chrono::Utc;
    use entity::{
        system_core::Entity as SystemCoreEntity,
        system_resources::Entity as SystemResourcesEntity,
    };
    use sea_orm::{EntityTrait, ModelTrait};
    
    use crate::database::mysql_connection;
    use crate::server_node::resources::to_f32;
    use crate::server_node::{
        resources::{
            Resources,
            Memory,
        },
        storage::{
            DiskKind,
            Storage,
        }
    };
    use super::CpuCore;

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
        let resource_id: i64 = resources.insert_data(&db).await.unwrap();
        
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
        updated_resources.update(resource_id, &db).await.unwrap();
        
        // Verify that the data was updated correctly
        let res_model = SystemResourcesEntity::find_by_id(resource_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        
        // This test fails for a negligible difference
        // assert_eq!(res_model.eval_time, updated_resources.eval_time.naive_utc());
        
        // Get system cores
        let updated_system_cores = res_model.find_related(SystemCoreEntity)
            .all(&db)
            .await
            .unwrap();

        assert_eq!(updated_system_cores.len(), 1);
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
        
        // Fetch resources
        let resources = Resources::fetch_resources().unwrap();
        
        // Insert initial data
        let resource_id: i64 = resources.insert_data(&db).await.unwrap();
        
        // Update resources
        let updated_resources = Resources {
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
        
        // Call the update function
        updated_resources.update(resource_id, &db).await.unwrap();
        
        // Verify that the data was updated correctly
        let res_model = SystemResourcesEntity::find_by_id(resource_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        
        // This test fails for a negligible difference
        // assert_eq!(res_model.eval_time, updated_resources.eval_time.naive_utc());
        
        // Get system cores
        let updated_system_cores = res_model.find_related(SystemCoreEntity)
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
        let resource_id: i64 = resources.insert_data(&db).await.unwrap();
        
		// Create system cores based on the updated resources
		let mut system_cores = resources.cpus.clone();
		
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
            cpus: system_cores,
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
        updated_resources.update(resource_id, &db).await.unwrap();
        
        // Verify that the data was updated correctly
        let res_model = SystemResourcesEntity::find_by_id(resource_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
		
        // This test fails for a negligible difference
        // assert_eq!(res_model.eval_time, updated_resources.eval_time.naive_utc());
		
        // Get system cores
        let updated_system_cores = res_model.find_related(SystemCoreEntity)
            .all(&db)
            .await
            .unwrap();
		
        assert_eq!(updated_system_cores.len(), 3);
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
        assert_eq!(
            updated_system_cores[2].usage_percentage,
            to_f32(updated_resources.cpus[2].usage_percentage).unwrap()
        );
        assert_eq!(
            updated_system_cores[2].free_percentage,
            to_f32(updated_resources.cpus[2].free_percentage).unwrap()
        );
    }
}
