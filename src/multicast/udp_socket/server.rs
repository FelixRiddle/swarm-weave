use std::error::Error;
use std::net::{UdpSocket, Ipv4Addr};
use std::str::FromStr;

use crate::config::env::{multicast_port, network_multicast_ip};

/// Start server
/// 
/// 
pub async fn start_server() -> Result<(), Box<dyn Error>> {
    // Local ip and port
    let ip = "0.0.0.0";
    let port = multicast_port();
    let location = format!("{ip}:{port}");
    
    let socket = UdpSocket::bind(&location)?;
    socket.set_multicast_loop_v4(true)?;
    
    // Network multicast
    let multicast_addr = Ipv4Addr::from_str(&network_multicast_ip())?;
    
    let interface_addr = Ipv4Addr::new(0, 0, 0, 0);
    socket.join_multicast_v4(&multicast_addr, &interface_addr)?;
    println!("UDP server started at: {location}");
    println!("UDP binded to {}", multicast_addr.to_string());
    
    loop {
        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                println!("Message received");
                
                match std::str::from_utf8(&buf[..amt]) {
                    Ok(msg) => println!("Received message from {}: {}", src, msg),
                    Err(e) => eprintln!("Error decoding message: {}", e),
                }
            }
            Err(e) => eprintln!("Error receiving message: {}", e),
        }
    }
}