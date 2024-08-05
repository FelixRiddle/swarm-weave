use std::env;

/// Retrieves the multicast IP address from the environment variable "MULTICAST_IP".
///
/// # Errors
///
/// Returns an error if the "MULTICAST_IP" environment variable is not set.
pub fn multicast_ip() -> Result<String, std::env::VarError> {
    env::var("MULTICAST_IP")
}

/// Retrieves the multicast port from the environment variable "MULTICAST_PORT".
///
/// # Errors
///
/// Returns an error if the "MULTICAST_PORT" environment variable is not set.
pub fn multicast_port() -> Result<String, std::env::VarError> {
    env::var("MULTICAST_PORT")
}