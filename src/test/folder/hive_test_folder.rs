use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

/// Hive test folder
/// 
/// Because it seems almost impossible to perform normal testing, we need to grow or roots,
/// and take finer control of the steps to test the p2p hive network configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HiveTestFolder {
    pub path: String,
}

impl Default for HiveTestFolder {
    fn default() -> Self {
        Self {
            path: ".cache/test/hive_test_folder".to_string(),
        }
    }
}

impl HiveTestFolder {
    pub fn new(path: String) -> Self {
        Self { path }
    }
    
    pub fn clean(&self) -> std::io::Result<()> {
        if Path::new(&self.path).exists() {
            fs::remove_dir_all(&self.path)?;
        }
        Ok(())
    }
    
	/// Create
	/// 
	/// Create if it doesn't exists
    pub fn create(&self) -> std::io::Result<()> {
        if !Path::new(&self.path).exists() {
            fs::create_dir_all(&self.path)?;
        }
        Ok(())
    }
}
