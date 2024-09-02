use std::error::Error;

use sea_orm::{Database, DatabaseConnection};

use crate::config::env::{mysql_database, mysql_host, mysql_password, mysql_port, mysql_username};

/// Mysql connection string
/// 
/// 
pub fn mysql_connection_string() -> String {
    let database = mysql_database();
    format!("mysql://{}:{}@{}:{}/{}", mysql_username(), mysql_password(), mysql_host(), mysql_port(), database)
}

/// Create MySQL connection
/// 
/// 
pub async fn mysql_connection() -> Result<DatabaseConnection, Box<dyn Error>> {
    let connection_url = &mysql_connection_string();
    Ok(Database::connect(connection_url).await?)
}

// Cannot run test because environment variables are not set.
// FIXME: This should work now, as I figured out how to fix it
#[cfg(test)]
mod tests {
    use sea_orm::DbErr;
	
    use super::*;
	
    /// Test the MySQL connection
    /// 
    /// Requires to have environment variables set
    #[tokio::test]
    async fn test_mysql_connection() {
        // Parse environment variables
        dotenv::dotenv().ok();
		
		// INFO: If you run it again, this will not do anything, I don't know why.
		// The expected behavior is to reset environment variables to whatever the .env file has
        // dotenv::dotenv().ok();
        
        let connection = mysql_connection().await.unwrap();
        
        assert!(connection.ping().await.is_ok());
        connection.clone().close().await.unwrap();
        
        assert!(
            matches!(
                connection.ping().await,
                Err(DbErr::ConnectionAcquire(_))
            )
        );
    }
}
