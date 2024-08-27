use actix_web::{Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Deserialize, Serialize)]
struct AppInfo {
    name: String,
    display_name: String,
    version: String,
    description: String,
    authors: Vec<String>,
    keywords: Vec<String>,
    categories: Vec<String>,
}

impl AppInfo {
    fn new() -> Result<Self, Box<dyn Error>> {
        let cargo_toml = fs::read_to_string("Cargo.toml")?;
        let cargo_toml: toml::Value = toml::from_str(&cargo_toml)?;
        
        let name = cargo_toml["package"]["name"].as_str().ok_or("Missing 'name' in Cargo.toml")?.to_string();
        let version = cargo_toml["package"]["version"].as_str().ok_or("Missing 'version' in Cargo.toml")?.to_string();
        let description = cargo_toml["package"]["description"].as_str().ok_or("Missing 'description' in Cargo.toml")?.to_string();
        
        let authors = if let Some(authors) = cargo_toml["package"]["authors"].as_array() {
            authors.iter().map(|author| author.as_str().unwrap_or("").to_string()).collect()
        } else {
            Vec::new()
        };
        
        let keywords = if let Some(keywords) = cargo_toml["package"]["keywords"].as_array() {
            keywords.iter().map(|keyword| keyword.as_str().unwrap_or("").to_string()).collect()
        } else {
            Vec::new()
        };

        let categories = if let Some(categories) = cargo_toml["package"]["categories"].as_array() {
            categories.iter().map(|category| category.as_str().unwrap_or("").to_string()).collect()
        } else {
            Vec::new()
        };
        
        Ok(AppInfo {
            name,
            display_name: String::from("Swarm weave"),
            version,
            description,
            authors,
            keywords,
            categories,
        })
    }
}

pub async fn app_info() -> impl Responder {
    match AppInfo::new() {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use actix_web::{
        test::{
            self,
            TestRequest
        },
        App,
        web,
        http::StatusCode
    };
    
    use super::{
        AppInfo,
        app_info,
    };
    
    #[actix_web::test]
    async fn test_status() {
        // Create a new service
        let app = test::init_service(App::new()
            .route("/app-info", web::get().to(app_info))).await;
        
        // Prepare a test request
        let req = TestRequest::get().uri("/app-info").to_request();
        
        // Send the request to the service and get the response
        let resp = test::call_service(&app, req).await;
        
        // Assert the response status code
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_body() {
        // Create a new service
        let app = test::init_service(App::new()
            .route("/app-info", web::get().to(app_info))).await;
        
        // Prepare a test request
        let req = TestRequest::get().uri("/app-info").to_request();
        
        // Assert the response body
        let app_info: AppInfo = test::call_and_read_body_json(&app, req).await;
        
        let cargo_toml = fs::read_to_string("Cargo.toml").unwrap();
        let cargo_toml: toml::Value = toml::from_str(&cargo_toml).unwrap();
        
        let name = cargo_toml["package"]["name"].as_str().ok_or("Missing 'name' in Cargo.toml").unwrap().to_string();
        let version = cargo_toml["package"]["version"].as_str().ok_or("Missing 'version' in Cargo.toml").unwrap().to_string();
        let description = cargo_toml["package"]["description"].as_str().ok_or("Missing 'description' in Cargo.toml").unwrap().to_string();
        
        // Add more assertions for other fields
        assert_eq!(app_info.name, name);
        assert_eq!(app_info.version, version);
        assert_eq!(app_info.description, description);
    }
}
