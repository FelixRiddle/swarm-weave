use std::error::Error;
use names::Generator;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use super::HiveFolderTestSuite;

/// Hive server node
/// 
/// 
#[derive(Debug, Serialize, Deserialize)]
pub struct HiveServerNode {
    pub ip: String,
    pub port: u16,
    pub node_name: String,
    pub test_suite: HiveFolderTestSuite,
    pub path: PathBuf,
}

impl HiveServerNode {
    /// Create hive server node
    /// 
    /// 
    pub fn new(test_suite: HiveFolderTestSuite, ip: String, port: u16) -> Result<Self, Box<dyn Error>> {
        // Generate random name
        let mut generator = Generator::default();
        let node_name = match generator.next() {
            Some(name) => name,
            None => return Err(String::from("Failed to generate node name").into()),
        };
        
        // Server node path
        let mut path = test_suite.path();
        fs::create_dir(&path)?;
        
        path.push(node_name.clone());
        fs::create_dir(&path)?;
        
        // Create server node
        let server_node = Self { ip, port, node_name, test_suite, path };
        
        // Create log path
        let path = server_node.get_log_folder();
        fs::create_dir(&path)?;
        
        Ok(server_node)
    }
    
    /// Save configuration
    /// 
    /// 
    pub fn save_config(&self) -> std::io::Result<()> {
        let config = serde_json::to_string(self)?;
        
        let config_file = self.path.join("config.json");
        
        std::fs::write(&config_file, config)?;
        Ok(())
    }
    
    /// Get log folder
    /// 
    /// 
    pub fn get_log_folder(&self) -> PathBuf {
        self.path.join("logs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::folder::hive_test_folder::HiveTestFolder;
    
    #[test]
    fn test_hive_server_node_creation() {
        let test_suite = HiveFolderTestSuite::new(HiveTestFolder::default());
        let server_node = HiveServerNode::new(test_suite.clone(), "127.0.0.1".to_string(), 8080).unwrap();
        
        assert!(server_node.ip == "127.0.0.1");
        assert!(server_node.port == 8080);
        assert!(server_node.node_name.len() > 0);
        assert!(server_node.path.to_str().unwrap().ends_with(&server_node.node_name));
        assert!(server_node.path.exists()); // Assert the path exists
        
        server_node.save_config().unwrap();
    }
}
