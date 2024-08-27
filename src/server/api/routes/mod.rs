use actix_web::{
    Scope,
    web
};

pub mod api;
pub mod app_info;

/// Routes main
/// 
/// 
pub fn main() -> Scope {
    web::scope("")
        .route("/app-info", web::get().to(app_info::app_info))
        .service(api::main())
}
