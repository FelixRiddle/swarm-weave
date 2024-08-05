use std::error::Error;
use std::net::{UdpSocket, Ipv4Addr};

/// Start server
/// 
/// 
pub async fn start_server() -> Result<(), Box<dyn Error>> {
    let ip = "0.0.0.0";
    let port = "3014";
    let location = format!("{ip}:{port}");
    
    let socket = UdpSocket::bind(&location)?;
    socket.set_multicast_loop_v4(true)?;
    
    let multicast_addr = Ipv4Addr::new(224, 0, 0, 1);
    let interface_addr = Ipv4Addr::new(0, 0, 0, 0);
    socket.join_multicast_v4(&multicast_addr, &interface_addr)?;
    println!("UDP server started at: {location}");

    loop {
        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                match std::str::from_utf8(&buf[..amt]) {
                    Ok(msg) => println!("Received message from {}: {}", src, msg),
                    Err(e) => eprintln!("Error decoding message: {}", e),
                }
            }
            Err(e) => eprintln!("Error receiving message: {}", e),
        }
    }
}