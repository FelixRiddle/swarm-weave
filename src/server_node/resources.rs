use chrono::{
    DateTime,
    Utc
};
use futures::StreamExt;
use std::error::Error;
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection,
    // DbBackend, DbErr, QueryFilter
};
use serde::{Deserialize, Serialize};
use sysinfo::{
    Disks, System
};
use entity::{
    // sea_orm_active_enums::Status,
    system_resources,
    system_core,
    system_memory,
    storage_device,
};
use super::storage::Storage;

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

/// Create system core instance
/// 
/// 
fn create_system_core_instance(cpu: &Cpu, system_resources_id: i64) -> Result<system_core::ActiveModel, Box<dyn Error>> {
    // Create system core
    let system_core_instance = system_core::ActiveModel {
        usage_percentage: ActiveValue::Set(to_f32(cpu.usage_percentage)?),
        free_percentage: ActiveValue::Set(to_f32(cpu.free_percentage)?),
        system_resource_id: ActiveValue::Set(Some(system_resources_id)),
        ..Default::default() // all other attributes are `NotSet`
    };
    
    Ok(system_core_instance)
}

/// Ram memory
/// 
/// 
#[derive(Deserialize, Serialize)]
pub struct Memory {
    pub total: u64,
    pub used: u64,
}

impl Resources {
    /// Fetch system resources and create a new Resources instance
    /// 
    /// 
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
            total: sys.total_memory(),
            used: sys.used_memory(),
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
    
    pub fn total_cores(&self) -> u32 {
        self.cpus.len() as u32
    }
    
    pub fn total_storage_usage_percentage(&self) -> f64 {
        self.storage.iter().map(|storage| storage.usage_percentage()).sum::<f64>() / self.storage.len() as f64
    }
    
    /// Insert data
    /// 
    /// 
    pub async fn insert_data(db: &DatabaseConnection) -> Result<(), Box<dyn Error>> {
        let resources = Resources::fetch_resources()?;
        
        // Create and insert resources
        let system_resources_instance = system_resources::ActiveModel {
            eval_time: ActiveValue::Set(resources.eval_time.naive_utc()),
           ..Default::default() // all other attributes are `NotSet`
        };
        let system_resources_id = system_resources_instance.insert(db).await?.id;
        
        // Create system core instances
        let system_core_instances: Result<Vec<system_core::ActiveModel>, Box<dyn Error>> = resources.cpus.iter().map(|cpu| {
            create_system_core_instance(cpu, system_resources_id)
        }).collect();
        
        let system_core_instances = system_core_instances?;
        
        for system_core_instance in system_core_instances {
            system_core_instance.insert(db).await?;
        }
        
        // // System memory instance
        // let system_memory_instance = system_memory::ActiveModel {
        //     total: ActiveValue::Set(resources.memory.total),
        //     used: ActiveValue::Set(resources.memory.used),
        //     free: ActiveValue::Set(resources.memory.free),
        //     system_resource_id: ActiveValue::Set(Some(system_resources_id)),
        //    ..Default::default()
        // };
        
        // // system_memory::Model::new(
        // //     resources.memory.total,
        // //     resources.memory.used,
        // //     resources.memory.free,
        // //     system_resources_id
        // // );
        // let system_memory_id = system_memory_instance.save(db).await?.id;
        
        // // Insert storage data
        // for storage in &resources.storage {
        //     let storage_instance = storage_device::Model::new(storage.name.clone(), storage.total, storage.used, storage.free, system_resources_id);
        //     storage_instance.save(db).await?;
        // }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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

    #[test]
    fn test_storage_usage_percentage() {
        let resources = Resources::fetch_resources().unwrap();
        assert!(resources.total_storage_usage_percentage() >= 0.0 && resources.total_storage_usage_percentage() <= 100.0);
    }
}
