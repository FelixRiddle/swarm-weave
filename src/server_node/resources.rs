use chrono::{
    DateTime,
    Utc
};
use std::error::Error;
use serde::{Deserialize, Serialize};
use sysinfo::{
    Disks, System
};

use super::storage::Storage;

#[derive(Deserialize, Serialize)]
pub struct Resources {
    pub cpus: Vec<Cpu>,
    pub memory: Memory,
    pub storage: Vec<Storage>,
    pub eval_time: DateTime<Utc>
}

#[derive(Deserialize, Serialize)]
pub struct Cpu {
    pub usage_percentage: f64,
    pub free_percentage: f64,
}

#[derive(Deserialize, Serialize)]
pub struct Memory {
    pub total: f64,
    pub used: f64,
    pub free: f64,
}

impl Resources {
    pub fn total_cores(&self) -> u32 {
        self.cpus.len() as u32
    }
    
    pub fn total_storage_usage_percentage(&self) -> f64 {
        self.storage.iter().map(|storage| storage.usage_percentage()).sum::<f64>() / self.storage.len() as f64
    }
    
    pub fn fetch_resources() -> Result<Resources, Box<dyn Error>> {
        let sys = System::new_all();
        
        let cpus = sys.cpus()
            .into_iter()
            .map(|cpu| Cpu {
                usage_percentage: cpu.cpu_usage() as f64,
                free_percentage: (1.0 - cpu.cpu_usage() as f64) * 100.0,
            })
            .collect();
        
        let memory = Memory {
            total: sys.total_memory() as f64,
            used: sys.used_memory() as f64,
            free: sys.available_memory() as f64,
        };
        
        // Storage
        let disks = Disks::new_with_refreshed_list();
        let mut storages = Vec::new();

        for disk in disks.list() {
            let storage = Storage::new(disk)?;
            
            storages.push(storage);
        }
        
        Ok(Resources {
            cpus,
            memory,
            storage: storages,
            eval_time: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fetch_resources() {
        let resources = Resources::fetch_resources().unwrap();
        assert!(resources.cpus.len() > 0);
        assert!(resources.memory.total > 0.0);
        assert!(!resources.storage.is_empty()); // Check if storage vector is not empty
    }

    #[test]
    fn test_total_cores() {
        let resources = Resources::fetch_resources().unwrap();
        assert!(resources.total_cores() > 0);
    }

    #[test]
    fn test_storage_usage_percentage() {
        let resources = Resources::fetch_resources().unwrap();
        assert!(resources.total_storage_usage_percentage() >= 0.0 && resources.total_storage_usage_percentage() <= 100.0);
    }
}
