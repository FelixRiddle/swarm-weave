// use actix_web::{Service, dev::ServiceRequest, Error, HttpError, HttpResponse};
// use futures::future::{ok, Ready};
// use sea_orm::DbConn;

// use crate::security::verify_token;

// // Define the authentication middleware
// pub struct AuthMiddleware {
//     db: DbConn,
// }

// impl AuthMiddleware {
//     pub fn new(db: DbConn) -> Self {
//         AuthMiddleware { db }
//     }
// }

// /// Implement the Service trait for the middleware
// impl Service for AuthMiddleware {
//     type Request = ServiceRequest;
//     type Response = HttpResponse;
//     type Error = Error;
//     type Future = Ready<Result<Self::Response, Self::Error>>;
    
//     fn call(&self, req: ServiceRequest) -> Self::Future {
//         // Extract the authentication token from the request headers
//         let token = req.headers().get("Authorization");
        
//         // If no token is present, return an error
//         if token.is_none() {
//             return ok(req.error_response(HttpError::Unauthorized()));
//         }
        
//         // Verify the token using your token verification logic
//         // For demonstration purposes, we'll assume a dummy verification function
//         if !verify_token(token.unwrap().to_str().unwrap()) {
//             return ok(req.error_response(HttpError::Unauthorized()));
//         }
        
//         // If the token is valid, allow the request to proceed
//         ok(req)
//     }
// }

// // Dummy token verification function
// fn verify_token(token: &str) -> bool {
//     // Replace this with your actual token verification logic
//     true
// }
