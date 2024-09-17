use actix_web::{web, HttpRequest, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use std::error::Error;
use reqwest::Client;

use crate::server_node::ServerNode;
// use crate::server_node::controller::ServerNodeController;

/// Server node
///
///
async fn get_server_node() -> impl Responder {
	match ServerNode::new() {
		Ok(server_node) => HttpResponse::Ok().json(server_node),
		Err(err) => {
			HttpResponse::InternalServerError().body(format!("Error creating ServerNode: {}", err))
		}
	}
}

#[derive(Deserialize, Serialize)]
pub struct LocationRequest {
	// Can be anything an ip, a domain, a url, etc...
	// The only conditions is that it's accessible
	location: String,
}

/// Get server node information
/// 
/// When a server node location is given, this function can be used to retrieve node information and insert it on our database.
/// 
/// TODO: Finish this function
pub async fn get_server_node_information(_req: HttpRequest, body: web::Json<LocationRequest>) -> Result<(), Box<dyn Error>> {
	let location = body.location.clone();

	// Process the location here
	let client = Client::new();
	let response = client
		.get(format!("{}{}", location, "/api/server_node"))
		.send()
		.await?;
	
	// Get server node information
	let server_info = response
		.text()
		.await?;
	
	let _server_node: ServerNode = serde_json::from_str(&server_info)?;
	
	// // Create server node
	// let server_node_controller = ServerNodeController::new_bare()?;
	
	Ok(())
}

/// Create server node
///
/// TODO: Get location information and store on the database
async fn post_location(_req: HttpRequest, body: web::Json<LocationRequest>) -> impl Responder {
	match get_server_node_information(_req, body).await {
		Ok(()) => {
			HttpResponse::Ok().body("Location processed successfully")
		}
		Err(_) => HttpResponse::InternalServerError().body("Failed to get the given location"),
	}
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
