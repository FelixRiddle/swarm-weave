use std::env;

/// Check if it's development mode
/// 
/// 
fn is_development() -> bool {
    if env::var("DEBUG").is_ok() {
        return true;
    } else {
        return false;
    }
}

/// Retrieves the multicast IP address from the environment variable "NETWORK_MULTICAST_IP".
///
/// If the "MULTICAST_IP" environment variable is not set, the default value "224.0.0.1" is returned.
pub fn network_multicast_ip() -> String {
    env::var("NETWORK_MULTICAST_IP").unwrap_or_else(|_| "224.0.0.1".to_string())
}

/// Test ipv4 multicast address
/// 
/// 
#[test]
fn test_ipv4_multicast() {
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    
    let multicast_addr = Ipv4Addr::from_str(&network_multicast_ip()).unwrap();
    assert!(multicast_addr.is_multicast());
}

/// Retrieves the multicast port from the environment variable "MULTICAST_PORT".
/// 
/// If the "MULTICAST_PORT" environment variable is not set, the default value is determined based on the development mode.
/// If it's a development build, the default value is "3000", otherwise it's "8082".
pub fn multicast_port() -> String {
    env::var("MULTICAST_PORT").unwrap_or_else(|_| {
        let is_dev = is_development();
        
        if is_dev {
            "3000".to_string()
        } else {
            "8082".to_string()
        }
    })
}

/// Rest server port
/// 
/// 
pub fn rest_server_port() -> String {
    env::var("PORT").unwrap_or_else(|_| {
        let is_dev = is_development();
        
        if is_dev {
            // Because I have many apps that run on other ports I keep adding one for each app
            "3014".to_string()
        } else {
            "8082".to_string()
        }
    })
}

/// Set debug variable
/// 
/// 
#[cfg(debug_assertions)]
pub fn set_debug() {
    env::set_var("DEBUG", "TRUE");
}
