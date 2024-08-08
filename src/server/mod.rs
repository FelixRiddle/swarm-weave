use actix_web::{App, web, HttpServer};
use std::error::Error;

use crate::config::env::server_port;

pub mod app_info;

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

/// Start rest server
/// 
/// 
pub async fn start_server(start_server_options: StartServerOptions) -> Result<(), Box<dyn Error>> {
    let location = start_server_options.location();
    
    // Start the Actix-web server
    HttpServer::new(|| {
        App::new()
            .route("/app-info", web::get().to(app_info::app_info)) 
    })
        .bind(location)?
        .run()
        .await?;
    
    Ok(())
}
