use chrono::{offset::LocalResult, DateTime, TimeZone, Utc};
use entity::{
	storage_device::{
		ActiveModel as StorageDevice,
		Model as StorageDeviceModel,
	},
	system_core::{ActiveModel as SystemCoreActiveModel, Model as SystemCoreModel},
	system_memory::{ActiveModel as SystemMemoryActiveModel, Model as SystemMemoryModel},
	system_resources::{
		ActiveModel as SystemResourcesActiveModel,
		Model as SystemResourcesModel,
	},
};
use sea_orm::{
	ActiveValue,
	IntoActiveModel
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use sysinfo::{Disks, System};

pub mod controller;
pub mod storage;
pub mod system_core;
pub mod system_memory;

use crate::model::FromActiveModel;

use storage::Storage;
use system_core::CpuCore as Cpu;
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
		storage_device_model: Vec<StorageDeviceModel>,
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
		for storage in storage_device_model.clone() {
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
