use actix_web::{middleware::Logger, web, App, HttpServer};
use async_trait::async_trait;
use env_logger::Env;
use sea_orm::DatabaseConnection;
use std::error::Error;

use crate::{config::env::server_port, database::mysql_connection};

pub mod middleware;
pub mod multicast;
pub mod routes;

/// Start server options
/// 
/// 
pub struct StartServerOptions {
    pub host: String,
    pub port: u16,
}

impl StartServerOptions {
    /// Create a controlled default function that returns an error on failure
    /// 
    /// 
    pub fn default_controlled() -> Result<Self, Box<dyn Error>> {
        let host = String::from("127.0.0.1");
        let port = server_port().parse::<u16>()?;
        
        Ok(StartServerOptions {
            host,
            port,
        })
    }
    
    /// Get location
    /// 
    /// 
    pub fn location(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// Implement a default
impl Default for StartServerOptions {
    fn default() -> Self {
        let host = String::from("127.0.0.1");
        let port = server_port().parse::<u16>().unwrap();
        
        StartServerOptions {
            host,
            port,
        }
    }
}

/// Create a state that holds the connection
/// 
/// 
#[derive(Clone)]
pub struct AppState {
    // Under the hood, a sqlx::Pool is created and owned by DatabaseConnection.
    pub db: DatabaseConnection
}

/// Implement a trait that creates a new state for each connection
/// 
/// 
#[async_trait]
pub trait CreateAppState {
    async fn create_state() -> Result<AppState, Box<dyn Error>>;
}

/// Implement the trait for the state
/// 
/// 
#[async_trait]
impl CreateAppState for AppState {
    async fn create_state() -> Result<AppState, Box<dyn Error>> {
        let db = mysql_connection().await?;
        
        Ok(AppState {
            db,
        })
    }
}

/// Start rest server
/// 
/// 
pub async fn start_server(start_server_options: StartServerOptions) -> Result<(), Box<dyn Error>> {
    let location = start_server_options.location();
    
    println!("Server running at {location}");
    
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    // Create state
    let state = AppState::create_state().await?;
    
    // Start the Actix-web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(routes::main()) 
    })
        .bind(location)?
        .run()
        .await?;
    
    Ok(())
}
