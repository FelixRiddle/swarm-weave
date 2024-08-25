pub mod hive_test_folder;
pub mod hive_folder_test_suite;

#[derive(Debug)]
pub struct TestFolder {
    pub path: String,
}

impl Default for TestFolder {
    fn default() -> Self {
        Self {
            path: ".cache/test".to_string(),
        }
    }
}

impl TestFolder {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}
