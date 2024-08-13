use sysinfo::{
    System,
    Disks
};

pub struct Resources {
    pub cpus: Vec<Cpu>,
    pub memory: Memory,
    pub storage: Storage,
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

impl Resources {
    pub fn total_cores(&self) -> u32 {
        self.cpus.len() as u32
    }
    
    pub fn storage_usage_percentage(&self) -> f64 {
        (self.storage.used / self.storage.total) * 100.0
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
        
        let mut total_storage = 0.0;
        let mut used_storage = 0.0;
        
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            total_storage += disk.total_space() as f64;
            used_storage += disk.available_space() as f64;
        }
        
        let storage = Storage {
            total: total_storage,
            used: used_storage,
        };
        
        Resources {
            cpus,
            memory,
            storage,
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
        assert!(resources.storage.total > 0.0);
    }

    #[test]
    fn test_total_cores() {
        let resources = Resources::fetch_resources();
        assert!(resources.total_cores() > 0);
    }

    #[test]
    fn test_storage_usage_percentage() {
        let resources = Resources::fetch_resources();
        assert!(resources.storage_usage_percentage() >= 0.0 && resources.storage_usage_percentage() <= 100.0);
    }
}
