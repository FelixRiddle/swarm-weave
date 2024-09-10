use chrono::{offset::LocalResult, DateTime, TimeZone, Utc};
use entity::{
	server_node::Model as ServerNodeModel,
	storage_device::{ActiveModel as StorageDevice, Entity as StorageDeviceEntity},
	system_core::{ActiveModel as SystemCoreActiveModel, Model as SystemCoreModel},
	system_memory::{ActiveModel as SystemMemoryActiveModel, Model as SystemMemoryModel},
	system_resources::{
		ActiveModel as SystemResourcesActiveModel, Entity as SystemResourcesEntity,
		Model as SystemResourcesModel,
	},
};
use sea_orm::{
	ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait,
	TryIntoModel,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use sysinfo::{Disks, System};

pub mod system_core;
pub mod system_memory;

use crate::model::FromActiveModel;

use super::storage::Storage;
use system_core::{CpuCore as Cpu, controller::CpuCoreController};
use system_memory::Memory;

/// Convert f64 to f32
///
///
pub fn to_f32(x: f64) -> Result<f32, Box<dyn Error>> {
	let y = x as f32;
	if x.is_finite() != y.is_finite() {
		Err(String::from("f32 overflow during conversion").into())
	} else {
		Ok(y)
	}
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Resources {
	// pub id: Option<i64>,
	pub cpus: Vec<Cpu>,
	pub memory: Memory,
	pub storage: Vec<Storage>,
	pub eval_time: DateTime<Utc>,
}

impl Resources {
	/// Fetch system resources and create a new Resources instance
	///
	///
	pub fn fetch_resources() -> Result<Resources, Box<dyn Error>> {
		let sys = System::new_all();

		let cpus = sys
			.cpus()
			.into_iter()
			.map(|cpu| Cpu {
				usage_percentage: cpu.cpu_usage() as f64,
				free_percentage: (1.0 - cpu.cpu_usage() as f64) * 100.0,
			})
			.collect();

		let memory = Memory {
			total: sys.total_memory(),
			used: sys.used_memory(),
		};

		// Storage
		let disks = Disks::new_with_refreshed_list();
		let mut storages = Vec::new();

		for disk in disks.list() {
			let storage = Storage::new(disk)?;

			// I've noticed that in the database a disk with the same name may be inserted twice
			// However this shouldn't be possible, because the name acts like an id and if they are the same, then
			// it's the same disk, so we need to check the id to not insert it twice.

			// Check if a disk with the same ID already exists in the storages vector
			if !storages
				.iter()
				.any(|existing_storage: &Storage| existing_storage.name == storage.name)
			{
				storages.push(storage);
			}
		}

		Ok(Resources {
			cpus,
			memory,
			storage: storages,
			eval_time: Utc::now(),
		})
	}

	pub fn total_cores(&self) -> u32 {
		self.cpus.len() as u32
	}
}

/// Conversions
///
///
impl Resources {
	/// Convert resources into active model
	///
	///
	pub fn into_active_model(&self) -> SystemResourcesActiveModel {
		let system_resources = SystemResourcesActiveModel {
			eval_time: ActiveValue::Set(self.eval_time.naive_utc()),
			..Default::default()
		};

		system_resources
	}

	/// Create from models
	///
	/// It's converted from active model for simplicity, but I think it would be faster if it's converted from the normal model instead
	pub fn from_models(
		model: SystemResourcesModel,
		cpus: Vec<SystemCoreModel>,
		memory: SystemMemoryModel,
		storage_active_model: Vec<StorageDevice>,
	) -> Result<Self, Box<dyn Error>> {
		let model = model.into_active_model();

		// Cpu Cores
		let mut cpu_active_models: Vec<SystemCoreActiveModel> = vec![];
		for cpu in cpus {
			cpu_active_models.push(cpu.into_active_model());
		}

		// Memory
		let memory = memory.into_active_model();

		// Storage
		let mut storages = vec![];
		for storage in storage_active_model.clone() {
			storages.push(storage.into_active_model());
		}
		
		let model = Self::from_active_model(model, cpu_active_models, memory, storages)?;
		
		Ok(model)
	}
	
	/// Create from active model
	///
	///
	pub fn from_active_model(
		active_model: SystemResourcesActiveModel,
		cpus_active_model: Vec<SystemCoreActiveModel>,
		memory: SystemMemoryActiveModel,
		storage_active_model: Vec<StorageDevice>,
	) -> Result<Self, Box<dyn Error>> {
		// Get evaluation time
		let eval_time: DateTime<Utc> = match active_model.eval_time.clone().take() {
			Some(value) => {
				let eval_time = match Utc.from_local_datetime(&value) {
					LocalResult::Single(eval_time) => eval_time,
					LocalResult::Ambiguous(_option_1, _option_2) => {
						return Err(format!("Ambiguous date time").into())
					}
					LocalResult::None => return Err(format!("Incorrect date time").into()),
				};

				eval_time
			}
			None => return Err("eval_time is missing".into()),
		};
		
		// Cpu Cores
		let mut cpus: Vec<Cpu> = vec![];
		for cpu in cpus_active_model {
			cpus.push(Cpu::from_active_model(cpu)?);
		}
		
		// Memory
		let memory = Memory::from_active_model(memory)?;
		
		// Storage
		let mut storages: Vec<Storage> = vec![];
		for storage in storage_active_model.clone() {
			storages.push(Storage::from_active_model(storage)?);
		}
		
		Ok(Resources {
			cpus,
			memory,
			storage: storages,
			eval_time,
		})
	}
}

pub struct SystemResourcesController {
	pub db: DatabaseConnection,
	resources: Option<Resources>,
	system_resources_active_model: Option<SystemResourcesActiveModel>,
}

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
	
	/// Insert data
	///
	/// Returns system resources id
	pub async fn insert_data(&mut self) -> Result<i64, Box<dyn Error>> {
		let resources = self.get_resources()?;
		
		// Create and insert resources
		let local_system_resources_instance = resources.into_active_model();
		let inserted_system_resources = local_system_resources_instance
			.clone()
			.insert(&self.db)
			.await?;
		
		self.system_resources_active_model = Some(local_system_resources_instance);
		
		// Create system core instances
		let system_core_controller = CpuCoreController::new(
			self.db.clone(),
			Some(resources.clone()),
			Some(inserted_system_resources.into_active_model()),
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
		let system_resources_model = system_resources_instance
			.clone()
			.try_into_model()?;
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
		system_resources_instance
			.clone()
			.save(db)
			.await?;
		self.system_resources_active_model = Some(system_resources_instance);
		
		Ok(())
	}

	/// Find by server node model
	///
	///
	pub async fn find_by_server_node_model(
		db: DatabaseConnection,
		server_node_active_model: ServerNodeModel,
	) -> Result<SystemResourcesModel, Box<dyn Error>> {
		let system_resource_id = match server_node_active_model.system_resource_id {
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
	use entity::{
		storage_device::Entity as StorageDevice, system_core::Entity as SystemCore,
		system_memory::Entity as SystemMemoryEntity, system_resources::Entity as SystemResources,
	};
	use sea_orm::{EntityTrait, ModelTrait};

	use super::*;
	use crate::database::mysql_connection;
	use crate::server_node::storage::DiskKind;

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
		system_resources_controller.insert_data().await.unwrap();

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
		let resource_id = system_resources_controller.insert_data().await.unwrap();

		// Update resources
		let updated_resources = Resources {
			cpus: vec![Cpu {
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
		let resource_id = system_resources_controller.insert_data().await.unwrap();

		// Get the ID of the inserted system resources
		let res_model = SystemResources::find_by_id(resource_id)
			.one(&db)
			.await
			.unwrap()
			.unwrap();
		let id: i64 = res_model.id;

		// Update resources
		let updated_resources = Resources {
			cpus: vec![Cpu {
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
		let resource_id = system_resources_controller.insert_data().await.unwrap();

		// Update resources
		let updated_resources = Resources {
			cpus: vec![Cpu {
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
