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
    pub fn new(test_suite: HiveFolderTestSuite, ip: String, port: u16) -> Result<Self, Box<dyn Error>> {
        // Generate random name
        let mut generator = Generator::default();
        let node_name = match generator.next() {
            Some(name) => name,
            None => return Err(String::from("Failed to generate node name").into()),
        };
        
        // Server node path
        let mut path = test_suite.path();
        path.push(node_name.clone());
        
        // Create server node
        let server_node = Self { ip, port, node_name, test_suite, path };
        server_node.create_dir()?;
        
        Ok(server_node)
    }
    
    fn create_dir(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.path)?;
        Ok(())
    }
    
    pub fn save_config(&self) -> std::io::Result<()> {
        let config = serde_json::to_string(self)?;
        std::fs::write(&self.path, config)?;
        Ok(())
    }
}
