use actix_web::{
    Scope,
    web
};

/// Main
/// 
/// 
pub fn main() -> Scope {
    web::scope("/")
}
