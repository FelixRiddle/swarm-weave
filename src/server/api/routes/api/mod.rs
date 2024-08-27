use actix_web::{
    Scope,
    web
};

pub mod server_node;

/// Main
/// 
/// 
pub fn main() -> Scope {
    web::scope("/api")
        .service(
            web::scope("/server-node")
                .service(server_node::main())
        )
}
