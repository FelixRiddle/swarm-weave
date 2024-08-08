use actix_web::{App, HttpServer, middleware::Logger};
use env_logger::Env;
use std::error::Error;

use crate::config::env::server_port;

pub mod middleware;
pub mod routes;

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
    
    println!("Server running at {location}");
    
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    // Start the Actix-web server
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(routes::main()) 
    })
        .bind(location)?
        .run()
        .await?;
    
    Ok(())
}
