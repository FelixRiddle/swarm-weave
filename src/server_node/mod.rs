use sysinfo::System;

pub mod resources;

pub use resources::Resources;

pub enum Location {
    IpAddress(IpAddress),
    DomainName(String),
}

pub struct IpAddress {
    pub address: String,
    pub port: u16,
}

pub struct ServerNode {
    pub id: u32,
    pub hostname: Option<String>,
    pub location: Location,
    pub status: ServerStatus,
    pub resources: Resources,
    pub system_info: SystemInfo,
}

pub enum ServerStatus {
    Online,
    Offline,
    Maintenance,
}

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
