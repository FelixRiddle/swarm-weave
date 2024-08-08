use std::error::Error;

use sea_orm::{Database, DatabaseConnection};

use crate::config::env::{mysql_database, mysql_host, mysql_password, mysql_port, mysql_username};

/// Create MySQL connection
/// 
/// 
pub async fn mysql_connection() -> Result<DatabaseConnection, Box<dyn Error>> {
    let url = format!("mysql://{}:{}@{}:{}/{}", mysql_username(), mysql_password(), mysql_host(), mysql_port(), mysql_database());
    println!("Url: {}", url);
    Ok(Database::connect(&url).await?)
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
