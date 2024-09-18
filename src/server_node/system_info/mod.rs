use entity::system_info::{ActiveModel as SystemInfoActiveModel, Model as SystemInfoModel};
use sea_orm::{ActiveValue, IntoActiveModel};
use serde::{Deserialize, Serialize};
use sysinfo::System;

pub mod controller;

#[derive(Clone, Deserialize, Serialize)]
pub struct SystemInfo {
	pub name: String,
	pub kernel_version: String,
	pub os_version: String,
	pub host_name: String,
}

impl SystemInfo {
	pub fn new() -> Self {
		Self {
			name: System::name().unwrap_or("Unknown".to_string()),
			kernel_version: System::kernel_version().unwrap_or("Unknown".to_string()),
			os_version: System::os_version().unwrap_or("Unknown".to_string()),
			host_name: System::host_name().unwrap_or("Unknown".to_string()),
		}
	}
}

/// Transformations
///
///
impl SystemInfo {
	/// Convert into active model
	///
	///
	pub fn into_active_model(self) -> SystemInfoActiveModel {
		SystemInfoActiveModel {
			name: ActiveValue::Set(self.name),
			kernel_version: ActiveValue::Set(Some(self.kernel_version)),
			os_version: ActiveValue::Set(self.os_version),
			hostname: ActiveValue::Set(self.host_name),
			..Default::default()
		}
	}

	/// Create from model
	///
	///
	pub fn from_model(model: SystemInfoModel) -> Option<Self> {
		let model = model.into_active_model();
		let model = Self::from_active_model(model);

		model
	}

	/// Create from active model
	///
	///
	pub fn from_active_model(active_model: SystemInfoActiveModel) -> Option<Self> {
		let name = active_model.name.clone().take()?;

		let kernel_version = active_model.kernel_version.clone().take();
		let kernel_version = match kernel_version {
			Some(kernel_version) => {
				let kernel_version = match kernel_version {
					Some(kernel_version) => kernel_version,
					None => String::from("Unknown"),
				};

				kernel_version
			}
			None => return None,
		};

		let os_version = active_model.os_version.clone().take()?;
		let host_name = active_model.hostname.clone().take()?;

		Some(Self {
			name,
			kernel_version,
			os_version,
			host_name,
		})
	}
}

impl Into<SystemInfo> for SystemInfoModel {
	fn into(self) -> SystemInfo {
		SystemInfo {
			name: self.name,
			kernel_version: self.kernel_version.unwrap_or("Unknown".to_string()),
			os_version: self.os_version,
			host_name: self.hostname,
		}
	}
}
