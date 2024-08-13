use actix_web::{
    Scope,
    web
};

pub mod server_node;

/// Main
/// 
/// 
pub fn main() -> Scope {
    web::scope("/")
        .service(
            web::scope("/server_node")
                .service(server_node::main())
        )
}
