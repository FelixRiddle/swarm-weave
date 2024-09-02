use entity::system_info::{
	Entity as SystemInfoEntity,
	ActiveModel as SystemInfoActiveModel,
	Model as SystemInfoModel,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use std::error::Error;
use sysinfo::System;

use crate::database::mysql_connection;

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
	
	pub fn into_active_model(self) -> SystemInfoActiveModel {
		SystemInfoActiveModel {
			name: ActiveValue::Set(self.name),
			kernel_version: ActiveValue::Set(Some(self.kernel_version)),
			os_version: ActiveValue::Set(self.os_version),
			hostname: ActiveValue::Set(self.host_name),
			..Default::default()
		}
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

#[derive(Clone)]
pub struct SystemInfoController {
    pub db: DatabaseConnection,
    pub system_info: SystemInfo,
}

impl SystemInfoController {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let db = mysql_connection().await?;
        let system_info = SystemInfo::new();
        
        Ok(Self { db, system_info })
    }
    
    pub async fn insert(self) -> Result<SystemInfoModel, Box<dyn Error>> {
        let result = self.system_info.clone().into_active_model().insert(&self.db).await?;
        
        Ok(result)
    }
    
    pub async fn update(self) -> Result<Self, Box<dyn Error>> {
        self.system_info.clone().into_active_model().update(&self.db).await?;
        
        Ok(self)
    }
    
    pub async fn find(&mut self, id: i64) -> Result<&mut Self, Box<dyn Error>> {
		let found_system_info: Option<SystemInfoModel> = SystemInfoEntity::find_by_id(id).one(&self.db).await?;
		let system_info: SystemInfo = match found_system_info {
			Some(model) => model.into(),
			None => return Err("System info not found".into()),
		};
		
		self.system_info = system_info;
		Ok(self)
	}
	
    pub async fn delete(self, id: i64) -> Result<Self, Box<dyn Error>> {
        let delete_result = SystemInfoEntity::delete_by_id(id).exec(&self.db).await?;
        assert_eq!(delete_result.rows_affected, 1);
        
        Ok(self)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_system_info_new() {
        let system_info = SystemInfo::new();
        assert!(system_info.name.len() > 0);
        assert!(system_info.kernel_version.len() > 0);
        assert!(system_info.os_version.len() > 0);
        assert!(system_info.host_name.len() > 0);
    }

    #[tokio::test]
    async fn test_system_info_controller_new() {
        let controller = SystemInfoController::new().await.unwrap();
        assert!(controller.system_info.name.len() > 0);
    }

    #[tokio::test]
    async fn test_system_info_controller_insert() {
        let controller = SystemInfoController::new().await.unwrap();
        let model = controller.clone().insert().await.unwrap();
        assert!(model.name.len() > 0);
        assert!(model.id > 0);
    }

    #[tokio::test]
    async fn test_system_info_controller_update() {
        let mut controller = SystemInfoController::new().await.unwrap();
        controller.system_info.name = "New name".to_string();
        controller = controller.update().await.unwrap();
        assert_eq!(controller.system_info.name, "New name");
    }

    #[tokio::test]
    async fn test_system_info_controller_find() {
        let controller = SystemInfoController::new().await.unwrap();
		
		// Insert model
        let model = controller.clone().insert().await.unwrap();
		
		// Find model
		let mut new_controller = controller.clone();
        let found_controller = new_controller.find(model.id).await.unwrap();
		
        assert_eq!(found_controller.system_info.name, controller.system_info.name);
    }

    #[tokio::test]
    async fn test_system_info_controller_delete() {
        let controller = SystemInfoController::new().await.unwrap();
        let model = controller.clone().insert().await.unwrap();
        let mut controller = controller.delete(model.id).await.unwrap();
        assert!(controller.find(model.id).await.is_err());
    }
}
