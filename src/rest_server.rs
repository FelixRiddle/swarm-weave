use actix_web::{App, web, HttpResponse, HttpServer, Responder};
use std::error::Error;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Server is up!")
}

/// Start rest server
/// 
/// 
pub async fn start_rest_server() -> Result<(), Box<dyn Error>> {
    // Start the Actix-web server
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
    })
        .bind("127.0.0.1:3015")?
        .run()
        .await?;
    
    Ok(())
}
