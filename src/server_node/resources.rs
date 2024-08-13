use sysinfo::{
    System,
    Disks
};

pub struct Resources {
    pub cpus: Vec<Cpu>,
    pub memory: Memory,
    pub storage: Vec<Storage>
}

pub struct Cpu {
    pub usage_percentage: f64,
    pub free_percentage: f64,
}

pub struct Memory {
    pub total: f64,
    pub used: f64,
    pub free: f64,
}

pub struct Storage {
    pub total: f64,
    pub used: f64,
}

impl Storage {
    pub fn usage_percentage(&self) -> f64 {
        (self.used as f64 / self.total as f64) * 100.0
    }
}

impl Resources {
    pub fn total_cores(&self) -> u32 {
        self.cpus.len() as u32
    }
    
    pub fn total_storage_usage_percentage(&self) -> f64 {
        self.storage.iter().map(|storage| storage.usage_percentage()).sum::<f64>() / self.storage.len() as f64
    }
    
    pub fn fetch_resources() -> Resources {
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
            let total = disk.total_space() as f64;
            let used = total - disk.available_space() as f64;

            let storage = Storage {
                total,
                used,
            };
            
            storages.push(storage);
        }
        
        Resources {
            cpus,
            memory,
            storage: storages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_resources() {
        let resources = Resources::fetch_resources();
        assert!(resources.cpus.len() > 0);
        assert!(resources.memory.total > 0.0);
        assert!(!resources.storage.is_empty()); // Check if storage vector is not empty
    }

    #[test]
    fn test_total_cores() {
        let resources = Resources::fetch_resources();
        assert!(resources.total_cores() > 0);
    }

    #[test]
    fn test_storage_usage_percentage() {
        let resources = Resources::fetch_resources();
        assert!(resources.total_storage_usage_percentage() >= 0.0 && resources.total_storage_usage_percentage() <= 100.0);
    }
}
