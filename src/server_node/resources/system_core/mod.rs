use entity::system_core::ActiveModel as SystemCoreActiveModel;
use sea_orm::{
	ActiveValue, TryIntoModel
};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub mod controller;

use crate::model::FromActiveModel;
use super::{to_f32, Resources};

/// CPU Core
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
