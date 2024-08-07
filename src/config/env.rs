use std::env;

/// Check if it's development mode
/// 
/// 
fn is_development() -> bool {
    if env::var("DEBUG").is_ok() {
        // Check if it's equal to 'false', lowercased
        if env::var("DEBUG").unwrap_or_default().to_lowercase() == "false" {
            return false;
        }
        
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
pub fn server_port() -> String {
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

// MySQL
/// Username
/// 
/// 
pub fn mysql_username() -> String {
    env::var("MYSQL_USERNAME").unwrap_or_else(|_| "root".to_string())
}

/// Password
/// 
/// 
pub fn mysql_password() -> String {
    env::var("MYSQL_PASSWORD").unwrap_or_else(|_| "".to_string())
}

/// Host
/// 
/// 
pub fn mysql_host() -> String {
    env::var("MYSQL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
}

/// Port
/// 
/// 
pub fn mysql_port() -> String {
    env::var("MYSQL_PORT").unwrap_or_else(|_| "3306".to_string())
}

/// Database
/// 
/// 
pub fn mysql_database() -> String {
    env::var("MYSQL_DATABASE").unwrap_or_else(|_| {
        // Check debug first
        if is_development() {
            return "perseverancia-development".to_string();
        } else {
            return "perseverancia-production".to_string();
        }
    })
}

/// Secret token
/// 
/// 
pub fn secret_token() -> String {
    env::var("SECRET_TOKEN").expect("Secret token is required, set it in the environment with the name 'SECRET_TOKEN'")
}

/// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_development() {
        // Set DEBUG environment variable to test different scenarios
        env::set_var("DEBUG", "TRUE");
        assert_eq!(is_development(), true);
        
        env::set_var("DEBUG", "false");
        assert_eq!(is_development(), false);
        
        env::remove_var("DEBUG");
        assert_eq!(is_development(), false);
    }

    #[test]
    fn test_network_multicast_ip() {
        env::set_var("NETWORK_MULTICAST_IP", "192.168.1.1");
        assert_eq!(network_multicast_ip(), "192.168.1.1");

        env::remove_var("NETWORK_MULTICAST_IP");
        assert_eq!(network_multicast_ip(), "224.0.0.1");
    }

    #[test]
    fn test_multicast_port() {
        env::set_var("MULTICAST_PORT", "4000");
        assert_eq!(multicast_port(), "4000");
        
        env::set_var("MULTICAST_PORT", "3000");
        assert_eq!(multicast_port(), "3000");
        
        #[cfg(not(debug_assertions))]
        {
            env::remove_var("DEBUG");
            assert_eq!(multicast_port(), "8082");
        }
    }

    #[test]
    fn test_server_port() {
        env::set_var("PORT", "5000");
        assert_eq!(server_port(), "5000");
        
        env::set_var("PORT", "3014");
        assert_eq!(server_port(), "3014");
        
        #[cfg(not(debug_assertions))]
        {
            env::remove_var("DEBUG");
            assert_eq!(rest_server_port(), "8082");
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_set_debug() {
        env::remove_var("DEBUG");
        set_debug();
        assert_eq!(env::var("DEBUG").unwrap(), "TRUE");
    }

    #[test]
    fn test_mysql_username() {
        env::set_var("MYSQL_USERNAME", "test_user");
        assert_eq!(mysql_username(), "test_user");

        env::remove_var("MYSQL_USERNAME");
        assert_eq!(mysql_username(), "root");
    }

    #[test]
    fn test_mysql_password() {
        env::set_var("MYSQL_PASSWORD", "test_password");
        assert_eq!(mysql_password(), "test_password");

        env::remove_var("MYSQL_PASSWORD");
        assert_eq!(mysql_password(), "");
    }

    #[test]
    fn test_mysql_host() {
        env::set_var("MYSQL_HOST", "192.168.1.2");
        assert_eq!(mysql_host(), "192.168.1.2");

        env::remove_var("MYSQL_HOST");
        assert_eq!(mysql_host(), "127.0.0.1");
    }

    #[test]
    fn test_mysql_port() {
        env::set_var("MYSQL_PORT", "3307");
        assert_eq!(mysql_port(), "3307");

        env::remove_var("MYSQL_PORT");
        assert_eq!(mysql_port(), "3306");
    }

    #[test]
    fn test_mysql_database() {
        env::set_var("DEBUG", "TRUE");
        env::set_var("MYSQL_DATABASE", "test_db");
        assert_eq!(mysql_database(), "test_db");

        env::remove_var("DEBUG");
        env::remove_var("MYSQL_DATABASE");
        assert_eq!(mysql_database(), "perseverancia-production");
    }

    #[test]
    #[should_panic]
    fn test_secret_token() {
        env::remove_var("SECRET_TOKEN");
        secret_token();
    }
}
