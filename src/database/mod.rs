use std::error::Error;

use sea_orm::{Database, DatabaseConnection};

use crate::config::env::{mysql_database, mysql_host, mysql_password, mysql_port, mysql_username};

/// Mysql connection string
/// 
/// 
pub fn mysql_connection_string() -> String {
    format!("mysql://{}:{}@{}:{}/{}", mysql_username(), mysql_password(), mysql_host(), mysql_port(), mysql_database())
}

/// Create MySQL connection
/// 
/// 
pub async fn mysql_connection() -> Result<DatabaseConnection, Box<dyn Error>> {
    Ok(Database::connect(&mysql_connection_string()).await?)
}

// Cannot run test because environment variables are not set.
// #[cfg(test)]
// mod tests {
//     use sea_orm::DbErr;

//     use super::*;
    
//     /// Test the MySQL connection
//     /// 
//     /// Requires to have environment variables set
//     #[tokio::test]
//     async fn test_mysql_connection() {
//         // Parse environment variables
//         dotenv::dotenv().ok();
        
//         let connection = mysql_connection().await.unwrap();
        
//         assert!(connection.ping().await.is_ok());
//         connection.clone().close().await.unwrap();
        
//         assert!(
//             matches!(
//                 connection.ping().await,
//                 Err(DbErr::ConnectionAcquire(_))
//             )
//         );
//     }
// }
