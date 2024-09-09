use entity::system_memory::ActiveModel as SystemMemoryActiveModel;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::model::FromActiveModel;

/// Ram memory
///
///
#[derive(Clone, Deserialize, Serialize)]
pub struct Memory {
	pub total: u64,
	pub used: u64,
}

impl Memory {
	/// Try into active model
	///
	///
	pub fn try_into_active_model(
		&self,
		system_resources_id: i64,
	) -> Result<SystemMemoryActiveModel, Box<dyn Error>> {
		let system_mem = SystemMemoryActiveModel {
			total: ActiveValue::Set(i64::try_from(self.total)?),
			used: ActiveValue::Set(i64::try_from(self.used)?),
			system_resource_id: ActiveValue::Set(Some(system_resources_id)),
			..Default::default()
		};
		
		Ok(system_mem)
	}
}

impl FromActiveModel<SystemMemoryActiveModel, Self> for Memory {
	fn from_active_model(active_model: SystemMemoryActiveModel) -> Result<Self, Box<dyn Error>> {
		// Memory
		let total = match active_model.total.clone().take() {
			Some(value) => value,
            None => return Err("Memory's total is missing".into()),
		};
		let used = match active_model.used.clone().take() {
            Some(value) => value,
            None => return Err("Memory's used is missing".into()),
        };
		let memory = Memory {
			total: u64::try_from(total)?,
			used: u64::try_from(used)?,
		};
		
		Ok(memory)
	}
}
