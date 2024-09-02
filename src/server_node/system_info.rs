use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Deserialize, Serialize)]
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
}
