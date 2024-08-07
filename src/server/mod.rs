use actix_web::{App, web, HttpResponse, HttpServer, Responder};
use std::error::Error;

use crate::config::env::server_port;

/// Start rest server
/// 
/// 
pub async fn start_server() -> Result<(), Box<dyn Error>> {
    let port = server_port();
    let host: &str = "127.0.0.1";
    let location = format!("{host}:{port}");
    
    // Start the Actix-web server
    HttpServer::new(|| {
        App::new()
            // .route("/", web::get().to(index))
    })
        .bind(location)?
        .run()
        .await?;
    
    Ok(())
}
