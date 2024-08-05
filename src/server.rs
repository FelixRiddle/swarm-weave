use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::error::Error;
use std::net::{UdpSocket, Ipv4Addr};
use std::thread;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Server is up!")
}

/// Start server
/// 
/// This function starts a server using Actix-web and performs multicast discovery.
/// It listens on the specified IP address and port for incoming requests.
/// Additionally, it sets up a separate thread to handle multicast communication.
/// 
/// # Returns
/// 
/// - `Result<Server, Box<dyn Error>>`: A result containing the Actix-web server instance if successful,
///   or an error if the server could not be started.
/// 
/// # Examples
/// 
/// ```rust
/// use actix_web::rt;
/// use swarm_weave::server;
/// 
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     let server = server::start_server().await?;
///     server.await
/// }
/// ```
pub async fn start_server() -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind("0.0.0.0:3014").expect("Failed to bind socket");
    socket.set_multicast_loop_v4(true).expect("Failed to set multicast loop");
    let multicast_addr = Ipv4Addr::new(239, 1, 1, 1);
    let interface_addr = Ipv4Addr::new(0, 0, 0, 0);
    socket.join_multicast_v4(&multicast_addr, &interface_addr).expect("Failed to join multicast group");
    
    // Run multicast communication in a separate thread
    thread::spawn(move || {
        loop {
            let mut buf = [0; 1024];
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    println!("Received message from {}: {}", src, std::str::from_utf8(&buf[..amt]).unwrap());
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                }
            }
        }
    });
    
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
