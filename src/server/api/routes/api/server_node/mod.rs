use actix_web::{web, HttpRequest, HttpResponse, Responder, Scope};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::server::api::AppState;
use crate::server_node::controller::ServerNodeController;
use crate::server_node::ServerNode;

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
pub async fn get_server_node_information(
	_req: HttpRequest,
	body: web::Json<LocationRequest>,
	data: web::Data<AppState>,
) -> Result<(), Box<dyn Error>> {
	let location = body.location.clone();

	// Process the location here
	let client = Client::new();
	let response = client
		.get(format!("{}{}", location, "/api/server_node"))
		.send()
		.await?;

	// Get server node information
	let server_info = response.text().await?;

	let server_node: ServerNode = serde_json::from_str(&server_info)?;

	// Get the database connection
	let db_conn = data.db.clone();

	// Create server node
	let mut server_node_controller = ServerNodeController::new_bare(db_conn)?;

	server_node_controller
		.insert_server_node(server_node)
		.await?;

	Ok(())
}

/// Create server node
///
/// Get location information and store on the database
async fn post_location(
	_req: HttpRequest,
	body: web::Json<LocationRequest>,
	data: web::Data<AppState>,
) -> impl Responder {
	match get_server_node_information(_req, body, data).await {
		Ok(()) => HttpResponse::Ok().body("Location processed successfully"),
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
	use crate::server_node::ServerStatus;
	use actix_web::{http::StatusCode, test, App};

	#[actix_web::test]
	async fn test_get_server_node() {
		let app = test::init_service(App::new().route("/", web::get().to(get_server_node))).await;
		let req = test::TestRequest::get().uri("/").to_request();
		let res = test::call_service(&app, req).await;

		assert_eq!(res.status(), StatusCode::OK);
		let body = test::read_body(res).await;
		let server_node: ServerNode = serde_json::from_slice(&body).unwrap();
		assert!(server_node.location.name.len() > 0);
		assert_eq!(server_node.status, ServerStatus::Online);
		assert!(server_node.system_info.name.len() > 0);
	}

	#[actix_web::test]
	async fn test_post_location() {
		let app = test::init_service(App::new().route("/", web::post().to(post_location))).await;
		let location_request = LocationRequest {
			location: String::from("http://example.com"),
		};
		let req = test::TestRequest::post()
			.uri("/")
			.set_json(&location_request)
			.to_request();
		let res = test::call_service(&app, req).await;

		assert_eq!(res.status(), StatusCode::OK);
		let body = test::read_body(res).await;
		let body_str = std::str::from_utf8(&body).unwrap();
		assert_eq!(body_str, "Location processed successfully");
	}

	#[actix_web::test]
	async fn test_post_location_invalid_request() {
		let app = test::init_service(App::new().route("/", web::post().to(post_location))).await;
		let req = test::TestRequest::post().uri("/").to_request();
		let res = test::call_service(&app, req).await;

		assert_eq!(res.status(), StatusCode::BAD_REQUEST);
	}

	#[actix_web::test]
	async fn test_post_location_internal_server_error() {
		// Mock an internal server error
		let app =
			test::init_service(App::new().route(
				"/",
				web::post().to(|| async {
					HttpResponse::InternalServerError().body("Internal Server Error")
				}),
			))
			.await;
		let location_request = LocationRequest {
			location: String::from("http://example.com"),
		};
		let req = test::TestRequest::post()
			.uri("/")
			.set_json(&location_request)
			.to_request();
		let res = test::call_service(&app, req).await;

		assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
	}
}
