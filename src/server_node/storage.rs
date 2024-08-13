use serde::{Deserialize, Serialize};
use sysinfo::{
    Disk, DiskKind as SysDiskKind,
};

#[derive(Serialize, Deserialize)]
pub enum DiskKind {
    HDD,
    SSD,
    Unknown,
}

#[derive(Serialize, Deserialize)]
pub struct Storage {
    pub total: f64,
    pub used: f64,
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
            total: disk.total_space() as f64,
            used: (disk.total_space() - disk.available_space()) as f64,
            kind: match disk.kind() {
                SysDiskKind::HDD => DiskKind::HDD,
                SysDiskKind::SSD => DiskKind::SSD,
                _ => DiskKind::Unknown,
            },
            name,
            is_removable: disk.is_removable(),
        })
    }
    
    pub fn usage_percentage(&self) -> f64 {
        (self.used / self.total) * 100.0
    }
    
    pub fn available_space(&self) -> f64 {
        self.total - self.used
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_usage_percentage() {
        let storage = Storage {
            total: 100.0,
            used: 50.0,
            kind: DiskKind::HDD,
            name: "sda1".to_string(),
            is_removable: true,
        };
        assert_eq!(storage.usage_percentage(), 50.0);
    }

    #[test]
    fn test_storage_available_space() {
        let storage = Storage {
            total: 100.0,
            used: 50.0,
            kind: DiskKind::HDD,
            name: "sda1".to_string(),
            is_removable: true,
        };
        assert_eq!(storage.available_space(), 50.0);
    }
}
