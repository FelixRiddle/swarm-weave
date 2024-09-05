use entity::storage_device::ActiveModel as StorageDeviceActiveModel;
use serde::{Deserialize, Serialize};
use sysinfo::{
    Disk, DiskKind as SysDiskKind,
};
use sea_orm::ActiveValue;

/// Disk kind
/// 
/// Sysinfo already has DiskKind however it's not serializable / deserializable
#[derive(Clone, Deserialize, Serialize)]
pub enum DiskKind {
    HDD,
    SSD,
    Unknown,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Storage {
    // In bytes
    pub total: u64,
    pub used: u64,
    pub kind: DiskKind,
    pub name: String,
    pub is_removable: bool,
}

impl Storage {
    pub fn new(disk: &Disk) -> Result<Self, std::io::Error> {
        let name = match disk.name().to_os_string().into_string() {
            Ok(name) => name,
            Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to convert disk name to string")),
        };
        
        Ok(Self {
            total: disk.total_space(),
            used: (disk.total_space() - disk.available_space()),
            kind: match disk.kind() {
                SysDiskKind::HDD => DiskKind::HDD,
                SysDiskKind::SSD => DiskKind::SSD,
                _ => DiskKind::Unknown,
            },
            name,
            is_removable: disk.is_removable(),
        })
    }
    
    pub fn usage_percentage(&self) -> f32 {
        (self.used as f32 / self.total as f32) * 100.0
    }
    
    pub fn available_space(&self) -> u64 {
        self.total - self.used
    }
	
	pub fn try_into_active_model(&self, system_resources_id: i64) -> Result<StorageDeviceActiveModel, Box<dyn std::error::Error>> {
		Ok(StorageDeviceActiveModel {
			name: ActiveValue::Set(self.name.clone()),
			total: ActiveValue::Set(i64::try_from(self.total)?),
			used: ActiveValue::Set(i64::try_from(self.used)?),
			system_resource_id: ActiveValue::Set(Some(system_resources_id)),
			is_removable: ActiveValue::Set(self.is_removable as i8),
			kind: ActiveValue::Set(serde_json::to_string(&self.kind)?),
			..Default::default()
		})
	}
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_usage_percentage() {
        let storage = Storage {
            total: 100,
            used: 50,
            kind: DiskKind::HDD,
            name: "sda1".to_string(),
            is_removable: true,
        };
        assert_eq!(storage.usage_percentage(), 50.0);
    }
    
    #[test]
    fn test_storage_available_space() {
        let storage = Storage {
            total: 100,
            used: 50,
            kind: DiskKind::HDD,
            name: "sda1".to_string(),
            is_removable: true,
        };
        assert_eq!(storage.available_space(), 50);
    }
}
