use chrono::{
    DateTime,
    Utc
};
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
            
            // I've noticed that in the database a disk with the same name may be inserted twice
            // However this shouldn't be possible, because the name acts like an id and if they are the same, then
            // it's the same disk, so we need to check the id to not insert it twice.
            
            // Check if a disk with the same ID already exists in the storages vector
            if !storages.iter().any(|existing_storage: &Storage| existing_storage.name == storage.name) {
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
    
    /// Insert data
    /// 
    /// 
    pub async fn insert_data(&self, db: &DatabaseConnection) -> Result<(), Box<dyn Error>> {
        
        // Create and insert resources
        let system_resources_instance = system_resources::ActiveModel {
            eval_time: ActiveValue::Set(self.eval_time.naive_utc()),
           ..Default::default() // all other attributes are `NotSet`
        };
        let system_resources_id = system_resources_instance.insert(db).await?.id;
        
        // Create system core instances
        let system_core_instances: Result<Vec<system_core::ActiveModel>, Box<dyn Error>> = self.cpus.iter().map(|cpu| {
            create_system_core_instance(cpu, system_resources_id)
        }).collect();
        let system_core_instances = system_core_instances?;
        for system_core_instance in system_core_instances {
            system_core_instance.insert(db).await?;
        }
        
        // System memory instance
        let system_memory_instance = system_memory::ActiveModel {
            total: ActiveValue::Set(i64::try_from(self.memory.total)?),
            used: ActiveValue::Set(i64::try_from(self.memory.used)?),
            system_resource_id: ActiveValue::Set(Some(system_resources_id)),
           ..Default::default()
        };
        system_memory_instance.insert(db).await?;
        
        // Insert storage data
        for storage in &self.storage {
            let storage_instance = storage_device::ActiveModel {
                name: ActiveValue::Set(storage.name.clone()),
                total: ActiveValue::Set(i64::try_from(storage.total)?),
                used: ActiveValue::Set(i64::try_from(storage.used)?),
                system_resource_id: ActiveValue::Set(Some(system_resources_id)),
                is_removable: ActiveValue::Set(storage.is_removable as i8),
                kind: ActiveValue::Set(serde_json::to_string(&storage.kind)?),
               ..Default::default()
            };
            storage_instance.save(db).await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use crate::database::mysql_connection;
    
    use sea_orm::EntityTrait;
    
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
    
    #[tokio::test]
    async fn test_insert_data() {
        // Set environment variables
        dotenv::dotenv().ok();
        
        // Initialize database connection
        let db = mysql_connection().await.unwrap();
        
        // Fetch resources
        let resources = Resources::fetch_resources().unwrap();
        
        // Call the insert_data function
        resources.insert_data(&db).await.unwrap();
        
        // Verify that data was inserted correctly
        let system_resources = system_resources::Entity::find()
            .all(&db)
            .await
            .unwrap();
        
        assert!(!system_resources.is_empty());
        
        let system_cores = system_core::Entity::find()
            .all(&db)
            .await
            .unwrap();
        
        assert!(!system_cores.is_empty());
        
        let system_memory = system_memory::Entity::find()
            .all(&db)
            .await
            .unwrap();
        
        assert!(!system_memory.is_empty());
        
        let storage_devices = storage_device::Entity::find()
            .all(&db)
            .await
            .unwrap();
        
        assert!(!storage_devices.is_empty());
    }
}
