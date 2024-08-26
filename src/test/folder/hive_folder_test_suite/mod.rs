use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;
use chrono::Utc;

use super::hive_test_folder::HiveTestFolder;

pub mod hive_server_node;

use hive_server_node::HiveServerNode;

/// Hive server node test context
/// 
/// 
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HiveFolderTestSuite {
    pub test_folder: HiveTestFolder,
    pub suite_date: String,
    pub short_id: String,
}

impl Default for HiveFolderTestSuite {
    fn default() -> Self {
        let test_folder = HiveTestFolder::default();
        let suite_date = Utc::now().format("%Y-%m-%d-%H-%M-%S").to_string();
        let short_id = nanoid::nanoid!(); // Generate a 6-character short id
        Self { test_folder, suite_date, short_id }
    }
}

impl HiveFolderTestSuite {
    pub fn new(test_folder: HiveTestFolder) -> Self {
        let suite_date = Utc::now().format("%Y-%m-%d-%H-%M-%S").to_string();
        let short_id = nanoid::nanoid!(6); // Generate a 6-character short id
        Self { test_folder, suite_date, short_id }
    }
    
    pub fn path(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.test_folder.path);
        path.push(format!("suite_{}_{}", &self.suite_date, &self.short_id));
        path
    }
    
    /// Create server node
    /// 
    /// 
    pub fn create_server_node(&self, ip: String, port: u16) -> Result<HiveServerNode, Box<dyn Error>> {
        HiveServerNode::new(self.clone(), ip, port)
    }
}
