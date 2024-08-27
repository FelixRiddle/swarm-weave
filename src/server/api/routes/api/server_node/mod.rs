use actix_web::{
    web, HttpRequest, HttpResponse, Responder, Scope
};
use serde::{Deserialize, Serialize};

use crate::server_node::ServerNode;

/// Server node
/// 
/// 
async fn get_server_node() -> impl Responder {
    match ServerNode::new(1) {
        Ok(server_node) => HttpResponse::Ok().json(server_node),
        Err(err) => HttpResponse::InternalServerError()
            .body(format!("Error creating ServerNode: {}", err)),
    }
}

#[derive(Deserialize, Serialize)]
struct LocationRequest {
    location: String,
}

/// Create server node
/// 
/// TODO: Get location information and store on the database
async fn post_location(_req: HttpRequest, body: web::Json<LocationRequest>) -> impl Responder {
    let location = body.location.clone();
    // Process the location here
    println!("Received location: {}", location);
    HttpResponse::Ok().body("Location received successfully")
}

/// Main
/// 
/// 
pub fn main() -> Scope {
    web::scope("")
        .route("", web::get().to(get_server_node))
        .route("", web::post().to(post_location))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    
    #[actix_web::test]
    async fn test_get_server_node() {
        let app = test::init_service(App::new().route("/", web::get().to(get_server_node))).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let res = test::call_service(&app, req).await;
        
        assert!(res.status().is_success());
        let body = test::read_body(res).await;
        let _server_node: ServerNode = serde_json::from_slice(&body).unwrap();
    }
}
