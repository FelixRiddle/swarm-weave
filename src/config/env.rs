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
    let password = match env::var("MYSQL_PASSWORD") {
        Ok(password) => {
            password
        },
        Err(_) => {
            String::from("")
        }
    };
    
    password
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

/// Get mysql database name
/// 
/// 
fn get_mysql_database_name() -> String {
    match env::var("MYSQL_DATABASE") {
        Ok(db_name) => return db_name.to_string(),
        Err(_) => { }
    };
    
    match env::var("MYSQL_DATABASE_NAME") {
        Ok(db_name) => return db_name.to_string(),
        Err(_) => { }
    };
    
    // Check debug first
    if is_development() {
        return "perseverancia-development".to_string();
    }
    
    "perseverancia-production".to_string()
}

/// Get test database name
/// 
/// 
fn get_test_database_name() -> String {
    match env::var("MYSQL_TEST_DATABASE") {
        Ok(db_name) => return db_name.to_string(),
        Err(_) => { }
    };
    
    match env::var("MYSQL_TEST_DATABASE_NAME") {
        Ok(db_name) => return db_name.to_string(),
        Err(_) => { }
    };
    
    "perseverancia-testing".to_string()
}

/// Database
/// 
/// 
pub fn mysql_database() -> String {
    // Check if it's testing
    if cfg!(test) {
        let db_name = get_test_database_name();
        
        return db_name;
    }
    
    // Get database name
    // TODO: Split development database from production
    get_mysql_database_name()
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
        let debug = env::var("DEBUG");
        
        // Set DEBUG environment variable to test different scenarios
        env::set_var("DEBUG", "TRUE");
        assert_eq!(is_development(), true);
        
        env::set_var("DEBUG", "false");
        assert_eq!(is_development(), false);
        
        env::remove_var("DEBUG");
        assert_eq!(is_development(), false);
        
        // Reset environment variables
        match debug {
            Ok(debug) => env::set_var("DEBUG", debug),
            Err(_) => {}
        };
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
        let port = env::var("PORT");
        
        env::set_var("PORT", "5000");
        assert_eq!(server_port(), "5000");
        
        env::set_var("PORT", "3014");
        assert_eq!(server_port(), "3014");
        
        #[cfg(not(debug_assertions))]
        {
            env::remove_var("DEBUG");
            assert_eq!(rest_server_port(), "8082");
        }
        
        // Reset environment variables
        match port {
            Ok(port) => {
                env::set_var("PORT", port);
            }
            Err(_) => {
            }
        };
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_set_debug() {
        let debug = env::var("DEBUG");
        
        env::remove_var("DEBUG");
        
        set_debug();
        assert_eq!(env::var("DEBUG").unwrap(), "TRUE");
        
        // Reset environment variables
        match debug {
            Ok(debug) => {
                env::set_var("DEBUG", debug);
            }
            Err(_) => {
            }
        };
    }

    #[test]
    fn test_mysql_username() {
        let username = env::var("USERNAME");
        
        env::set_var("MYSQL_USERNAME", "test_user");
        assert_eq!(mysql_username(), "test_user");

        env::remove_var("MYSQL_USERNAME");
        assert_eq!(mysql_username(), "root");
        
        match username {
            Ok(username) => {
                env::set_var("MYSQL_USERNAME", username);
            }
            Err(_) => {
            }
        };
    }
    
    #[test]
    fn test_mysql_password() {
        dotenv::dotenv().ok();
        let original_password = mysql_password();
        
        env::set_var("MYSQL_PASSWORD", "test_password");
        assert_eq!(mysql_password(), "test_password");
        
        // I think rust shares environment variables between tests, thought it didn't
        env::set_var("MYSQL_PASSWORD", "");
        assert_eq!(mysql_password(), "");
        
        // Use dotenv to assert the password is set correctly
        env::set_var("MYSQL_PASSWORD", &original_password);
        assert_eq!(mysql_password(), original_password);
    }
    
    #[test]
    fn test_mysql_host() {
        let host = env::var("MYSQL_HOST");
        
        env::set_var("MYSQL_HOST", "192.168.1.2");
        assert_eq!(mysql_host(), "192.168.1.2");

        env::remove_var("MYSQL_HOST");
        assert_eq!(mysql_host(), "127.0.0.1");
        
        match host {
            Ok(port) => {
                env::set_var("MYSQL_HOST", port);
            }
            Err(_) => {
            }
        };
    }
    
    #[test]
    fn test_mysql_port() {
        let port = env::var("PORT");
        
        env::set_var("MYSQL_PORT", "3307");
        assert_eq!(mysql_port(), "3307");

        env::remove_var("MYSQL_PORT");
        assert_eq!(mysql_port(), "3306");
        
        match port {
            Ok(port) => {
                env::set_var("PORT", port);
            }
            Err(_) => {
            }
        };
    }
    
    #[test]
    fn test_mysql_database() {
        // Get environment variables
        let debug_mode = env::var("DEBUG");
        let mysql_db = env::var("MYSQL_DATABASE");
        let mysql_database_name = env::var("MYSQL_DATABASE_NAME");
        
        // Test database name
        assert_eq!(mysql_database(), get_test_database_name());
        
        // Reset environment variables
        match debug_mode {
            Ok(debug_mode) => {
                env::set_var("DEBUG", debug_mode);
            }
            Err(_) => {}
        };
        match mysql_db {
            Ok(mysql_db) => {
                env::set_var("MYSQL_DATABASE", mysql_db);
            }
            Err(_) => {}
        };
        match mysql_database_name {
            Ok(mysql_database_name) => {
                env::set_var("MYSQL_DATABASE_NAME", mysql_database_name);
            }
            Err(_) => {}
        };
    }
}
