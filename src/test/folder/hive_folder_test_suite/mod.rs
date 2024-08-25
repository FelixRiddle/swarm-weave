use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;
use uuid::Uuid;

use super::hive_test_folder::HiveTestFolder;

pub mod hive_server_node;

use hive_server_node::HiveServerNode;

/// Hive server node test context
/// 
/// 
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HiveFolderTestSuite {
    pub test_folder: HiveTestFolder,
    pub suite_uuid: String,
}

impl HiveFolderTestSuite {
    pub fn new(test_folder: HiveTestFolder) -> Self {
        let suite_uuid = Uuid::new_v4().to_string();
        Self { test_folder, suite_uuid }
    }

    pub fn path(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.test_folder.path);
        path.push(format!("suite_{}", &self.suite_uuid));
        path
    }
    
    pub fn create_server_node(&self, ip: String, port: u16) -> Result<HiveServerNode, Box<dyn Error>> {
        HiveServerNode::new(self.clone(), ip, port)
    }
}
