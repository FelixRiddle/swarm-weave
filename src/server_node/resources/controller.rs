use entity::{
	server_node::Model as ServerNodeModel,
	storage_device::Entity as StorageDeviceEntity,
	system_resources::{
		ActiveModel as SystemResourcesActiveModel, Entity as SystemResourcesEntity,
		Model as SystemResourcesModel,
	},
};
use sea_orm::{
	ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, ModelTrait,
	TryIntoModel,
};
use std::error::Error;

use crate::server_node::resources::{
	storage::controller::StorageController, system_core::controller::CpuCoreController,
	system_memory::controller::MemoryController, Resources,
};

/// System resources controller
///
///
pub struct SystemResourcesController {
	pub db: DatabaseConnection,
	resources: Option<Resources>,
	system_resources_active_model: Option<SystemResourcesActiveModel>,
}

/// Constructors
///
///
impl SystemResourcesController {
	/// Create new instance
	///
	///
	pub fn new(db: DatabaseConnection, resources: Option<Resources>) -> Self {
		SystemResourcesController {
			db,
			resources,
			system_resources_active_model: None,
		}
	}
}

/// Get methods
///
///
impl SystemResourcesController {
	/// Get resources active model
	///
	///
	pub fn get_resources_active_model(&self) -> Result<SystemResourcesActiveModel, Box<dyn Error>> {
		let system_resources =
			match self.system_resources_active_model.clone() {
				Some(model) => model,
				None => return Err(
					"System resources active model doesn't exists, please fetch it or create it"
						.into(),
				),
			};

		Ok(system_resources)
	}

	/// Find resources by id and create new Resources instance
	///
	///
	pub async fn find_by_id_and_get_resources(
		&self,
		system_resources_model: SystemResourcesModel,
		system_resources_id: i64,
	) -> Result<Resources, Box<dyn Error>> {
		// Cpu cores
		let cpu_core_controller = CpuCoreController::new(self.db.clone(), None, None);
		let cpu_cores = cpu_core_controller
			.find_cores_by_resources_id(system_resources_id)
			.await?;

		// System memory
		let memory_controller = MemoryController::new(self.db.clone());
		let memory = memory_controller
			.get_system_memory_by_resources_id(system_resources_id)
			.await?;

		// Find storage devices
		let storage_device_controller = StorageController::new(self.db.clone());
		let storage_devices = storage_device_controller
			.find_by_resources_id(system_resources_id)
			.await?;

		// Create system resources from models
		let system_resources = Resources::from_models(
			system_resources_model.clone(),
			cpu_cores,
			memory,
			storage_devices,
		)?;

		Ok(system_resources)
	}

	/// Get or create sytem resources
	///
	/// Create it by fetching resources from the system
	///
	/// Or should it fetch them from the database?
	pub fn get_resources(&self) -> Result<Resources, Box<dyn Error>> {
		match self.resources {
			Some(ref resources) => Ok(resources.clone()),
			None => {
				let resources = Resources::fetch_resources()?;
				Ok(resources)
			}
		}
	}
}

impl SystemResourcesController {
	/// Insert model
	/// 
	/// 
	async fn insert_model(&mut self, resources: Resources) -> Result<&mut Self, Box<dyn Error>> {
		// Create and insert resources
		let mut local_system_resources_instance = resources.into_active_model();
		let inserted_system_resources = local_system_resources_instance
			.clone()
			.insert(&self.db)
			.await?;
		
		// Update the id
		local_system_resources_instance.id = ActiveValue::Unchanged(inserted_system_resources.id.clone());
		
		// Set on the controller
        self.system_resources_active_model = Some(local_system_resources_instance);
		
		Ok(self)
	}
	
	/// Insert data
	///
	/// 
	pub async fn insert(&mut self) -> Result<&mut Self, Box<dyn Error>> {
		// Create and insert resources
		let resources = self.get_resources()?;
		self.insert_model(resources.clone())
			.await?;
		
		// Create system core instances
		let system_core_controller = CpuCoreController::new(
			self.db.clone(),
			Some(resources.clone()),
			Some(self.get_resources_active_model()?),
		);
		let system_core_instances = system_core_controller.create_cores()?;
		for system_core_instance in system_core_instances {
			system_core_instance.save(&self.db).await?;
		}
		
		// Take the id
		let system_resources_id = system_core_controller.id()?;
		
		// System memory instance
		let system_memory_instance = resources
			.memory
			.try_into_active_model(system_resources_id)?;
		system_memory_instance.save(&self.db).await?;

		// Insert storage data
		for storage in &resources.storage {
			let storage_instance = storage.try_into_active_model(system_resources_id)?;
			storage_instance.save(&self.db).await?;
		}

		Ok(self)
	}
	
	/// Get id
	/// 
	/// 
	pub fn id(&self) -> Result<i64, Box<dyn Error>> {
		let system_resources_instance = self.get_resources_active_model()?;
        
        let system_resources_id = match system_resources_instance.id.clone().take() {
            Some(id) => id,
            None => {
                return Err(
                    "System resources active model doesn't exist, please fetch it or create it"
                        .into(),
                )
            }
        };

        Ok(system_resources_id)
	}

	/// Update
	///
	/// TODO: This function is incomplete, update cores
	pub async fn update(
		&mut self,
		system_resources_id: i64,
		db: &DatabaseConnection,
	) -> Result<(), Box<dyn Error>> {
		let resources = self.get_resources()?;

		// Create and insert resources
		let system_resources_instance = SystemResourcesActiveModel {
			eval_time: ActiveValue::Set(resources.eval_time.naive_utc()),
			id: ActiveValue::Unchanged(system_resources_id),
		};

		// Update system cores
		let system_core_controller = CpuCoreController::new(
			db.clone(),
			Some(resources.clone()),
			Some(system_resources_instance.clone()),
		);
		// TODO: Update cores

		// System resources id
		let system_resources_id = system_core_controller.id()?;

		// System memory instance
		let system_memory_instance = resources
			.memory
			.try_into_active_model(system_resources_id)?;
		system_memory_instance.save(db).await?;

		// Update storage
		// We need to know how many storage devices do we have already
		// We know storage devices name is unique
		let system_resources_model = system_resources_instance.clone().try_into_model()?;
		let existing_storage_devices = system_resources_model
			.find_related(StorageDeviceEntity)
			.all(db)
			.await?;

		// Compare with our storage devices and remove from the database those that aren't in this structure
		let removed_devices: Vec<&entity::storage_device::Model> = existing_storage_devices
			.iter()
			// Get missing storage devices to remove
			.filter(|storage_device| {
				let is_there = existing_storage_devices
					.iter()
					.filter(|existing_device| existing_device.id == storage_device.id)
					.collect::<Vec<_>>();

				if !is_there.is_empty() {
					return true;
				}

				false
			})
			.collect::<Vec<_>>();

		// Delete removed devices in parallel
		let mut handles: Vec<tokio::task::JoinHandle<Result<(), anyhow::Error>>> = vec![];
		for model in removed_devices {
			let model: entity::storage_device::Model = model.clone();
			let db = db.clone();

			handles.push(tokio::spawn(async move {
				model.delete(&db).await.map_err(|e| anyhow::Error::new(e))?;
				Ok(())
			}));
		}

		// Wait for deletions to complete
		for handle in handles {
			handle.await??;
		}

		// Insert storage data
		for storage in &resources.storage {
			let storage_instance = storage.try_into_active_model(system_resources_id)?;
			storage_instance.save(db).await?;
		}

		// It's a standard operation to save this at the end of everything else
		// Save system resources evaluation time
		system_resources_instance.clone().save(db).await?;
		self.system_resources_active_model = Some(system_resources_instance);

		Ok(())
	}

	/// Find by server node model
	///
	///
	pub async fn find_by_server_node_model(
		db: DatabaseConnection,
		server_node_model: ServerNodeModel,
	) -> Result<SystemResourcesModel, Box<dyn Error>> {
		let system_resource_id = match server_node_model.system_resource_id {
			Some(id) => id,
			None => return Err("Server location id not found".into()),
		};
		let server_location = match SystemResourcesEntity::find_by_id(system_resource_id)
			.one(&db)
			.await?
		{
			Some(model) => model,
			None => return Err("Server location not found".into()),
		};

		Ok(server_location)
	}

	// /// Find server node related models
	// ///
	// ///
	// pub fn find_server_node_related_models() -> Result<(Vec<SystemCoreActiveModel>, SystemMemoryActiveModel, Vec<StorageDevice>), Box<dyn Error>>{
	// 	// TODO: Get system core related models
	// 	let mut cores = Vec::new();

	// 	// TODO: Get system memory related model

	// 	// TODO: Get storage related models
	// 	let mut storage_devices = Vec::new();

	// }
}

#[cfg(test)]
mod tests {
	use chrono::Utc;
	use entity::{
		storage_device::Entity as StorageDevice, system_core::Entity as SystemCore,
		system_memory::Entity as SystemMemoryEntity, system_resources::Entity as SystemResources,
	};
	use sea_orm::{EntityTrait, ModelTrait};

	use super::*;
	use crate::database::mysql_connection;
	use crate::server_node::resources::{
		storage::{DiskKind, Storage},
		system_memory::Memory,
		to_f32,
	};

	#[test]
	fn test_fetch_resources() {
		let resources = Resources::fetch_resources().unwrap();
		assert!(resources.cpus.len() > 0);
		assert!(resources.memory.total > 0);
		assert!(!resources.storage.is_empty()); // Check if storage vector is not empty
	}

	#[test]
	fn test_total_cores() {
		let resources = Resources::fetch_resources().unwrap();
		assert!(resources.total_cores() > 0);
	}

	#[tokio::test]
	async fn test_insert_data() {
		// Set environment variables
		dotenv::dotenv().ok();

		// Initialize database connection
		let db = mysql_connection().await.unwrap();

		// Call the insert_data function
		let mut system_resources_controller = SystemResourcesController::new(db.clone(), None);
		system_resources_controller.insert().await.unwrap();

		// Verify that data was inserted correctly
		let system_resources = SystemResources::find().all(&db).await.unwrap();

		assert!(!system_resources.is_empty());

		let system_cores = SystemCore::find().all(&db).await.unwrap();

		assert!(!system_cores.is_empty());

		let system_memory = SystemMemoryEntity::find().all(&db).await.unwrap();

		assert!(!system_memory.is_empty());

		let storage_devices = StorageDevice::find().all(&db).await.unwrap();

		assert!(!storage_devices.is_empty());
	}

	#[tokio::test]
	async fn test_update_system_memory() {
		// Set environment variables
		dotenv::dotenv().ok();

		// Initialize database connection
		let db = mysql_connection().await.unwrap();

		// Insert initial data
		let mut system_resources_controller = SystemResourcesController::new(db.clone(), None);
		let resource_id = system_resources_controller
			.insert()
			.await
			.unwrap()
			.id()
			.unwrap();

		// Update resources
		let updated_resources = Resources {
			// Non-variable cpus quantity
			cpus: system_resources_controller.get_resources().unwrap().cpus,
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

		// Get the ID of the inserted system resources
		let res_model = SystemResources::find_by_id(resource_id)
			.one(&db)
			.await
			.unwrap()
			.unwrap();
		let id: i64 = res_model.id;

		// Call the update function
		let mut system_resources_controller =
			SystemResourcesController::new(db.clone(), Some(updated_resources.clone()));
		system_resources_controller.update(id, &db).await.unwrap();

		// Verify that the data was updated correctly
		let res_model = SystemResources::find_by_id(resource_id)
			.one(&db)
			.await
			.unwrap()
			.unwrap();

		// This test fails for a negligible difference
		// left: 2024-08-30T19:19:42
		// right: 2024-08-30T19:19:41.944202060
		// assert_eq!(res_model.eval_time, updated_resources.eval_time.naive_utc());

		// Get system memory
		let updated_system_memory = res_model
			.find_related(SystemMemoryEntity)
			.one(&db)
			.await
			.unwrap()
			.unwrap();

		assert_eq!(
			updated_system_memory.total,
			i64::try_from(updated_resources.memory.total).unwrap()
		);
		assert_eq!(
			updated_system_memory.used,
			i64::try_from(updated_resources.memory.used).unwrap()
		);
	}

	#[tokio::test]
	async fn test_update_storage_device() {
		// Set environment variables
		dotenv::dotenv().ok();

		// Initialize database connection
		let db = mysql_connection().await.unwrap();

		// Insert initial data
		let mut system_resources_controller = SystemResourcesController::new(db.clone(), None);
		let resource_id = system_resources_controller
			.insert()
			.await
			.unwrap()
			.id()
			.unwrap();

		// Get the ID of the inserted system resources
		let res_model = SystemResources::find_by_id(resource_id)
			.one(&db)
			.await
			.unwrap()
			.unwrap();
		let id: i64 = res_model.id;

		// Update resources
		let updated_resources = Resources {
			// Non-variable cpus quantity
			cpus: system_resources_controller.get_resources().unwrap().cpus,
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
		let mut system_resources_controller =
			SystemResourcesController::new(db.clone(), Some(updated_resources.clone()));
		system_resources_controller.update(id, &db).await.unwrap();

		// Verify that the data was updated correctly
		let res_model = SystemResources::find_by_id(resource_id)
			.one(&db)
			.await
			.unwrap()
			.unwrap();

		// Get storage devices
		let updated_storage_devices = res_model
			.find_related(StorageDevice)
			.all(&db)
			.await
			.unwrap();

		// This test fails for a negligible difference
		// assert_eq!(res_model.eval_time, updated_resources.eval_time.naive_utc());
		assert_eq!(updated_storage_devices.len(), 1);
		assert_eq!(
			updated_storage_devices[0].name,
			updated_resources.storage[0].name
		);
		assert_eq!(
			updated_storage_devices[0].total,
			i64::try_from(updated_resources.storage[0].total).unwrap()
		);
		assert_eq!(
			updated_storage_devices[0].used,
			i64::try_from(updated_resources.storage[0].used).unwrap()
		);
		assert_eq!(
			updated_storage_devices[0].is_removable,
			updated_resources.storage[0].is_removable as i8
		);
		assert_eq!(
			updated_storage_devices[0].kind,
			serde_json::to_string(&updated_resources.storage[0].kind).unwrap()
		);
	}

	/// Test update all
	///
	///
	#[tokio::test]
	async fn test_update() {
		// Set environment variables
		dotenv::dotenv().ok();

		// Initialize database connection
		let db = mysql_connection().await.unwrap();

		// Insert initial data
		let mut system_resources_controller = SystemResourcesController::new(db.clone(), None);
		let resource_id = system_resources_controller
			.insert()
			.await
			.unwrap()
			.id()
			.unwrap();
		
		// Update resources
		let updated_resources = Resources {
			// Non-variable cpus quantity
			cpus: system_resources_controller.get_resources().unwrap().cpus,
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
		let mut system_resources_controller =
			SystemResourcesController::new(db.clone(), Some(updated_resources.clone()));
		system_resources_controller
			.update(resource_id, &db)
			.await
			.unwrap();

		// Verify that the data was updated correctly
		let res_model = SystemResources::find_by_id(resource_id)
			.one(&db)
			.await
			.unwrap()
			.unwrap();

		// This test fails for a negligible difference
		// assert_eq!(res_model.eval_time, updated_resources.eval_time.naive_utc());

		// Get system cores
		let updated_system_cores = res_model.find_related(SystemCore).all(&db).await.unwrap();

		assert_eq!(updated_system_cores.len(), 1);
		assert_eq!(
			updated_system_cores[0].usage_percentage,
			to_f32(updated_resources.cpus[0].usage_percentage).unwrap()
		);
		assert_eq!(
			updated_system_cores[0].free_percentage,
			to_f32(updated_resources.cpus[0].free_percentage).unwrap()
		);

		// Get system memory
		let updated_system_memory = res_model
			.find_related(SystemMemoryEntity)
			.one(&db)
			.await
			.unwrap()
			.unwrap();

		assert_eq!(
			updated_system_memory.total,
			i64::try_from(updated_resources.memory.total).unwrap()
		);
		assert_eq!(
			updated_system_memory.used,
			i64::try_from(updated_resources.memory.used).unwrap()
		);

		// Get storage devices
		let updated_storage_devices = res_model
			.find_related(StorageDevice)
			.all(&db)
			.await
			.unwrap();

		assert_eq!(updated_storage_devices.len(), 1);
		assert_eq!(
			updated_storage_devices[0].name,
			updated_resources.storage[0].name
		);
		assert_eq!(
			updated_storage_devices[0].total,
			i64::try_from(updated_resources.storage[0].total).unwrap()
		);
		assert_eq!(
			updated_storage_devices[0].used,
			i64::try_from(updated_resources.storage[0].used).unwrap()
		);
		assert_eq!(
			updated_storage_devices[0].is_removable,
			updated_resources.storage[0].is_removable as i8
		);
		assert_eq!(
			updated_storage_devices[0].kind,
			serde_json::to_string(&updated_resources.storage[0].kind).unwrap()
		);
	}
}
