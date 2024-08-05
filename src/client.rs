use std::{error::Error, net::{Ipv4Addr, UdpSocket}, str::FromStr};

use crate::config::env::{multicast_port, network_multicast_ip};

/// Send messages to the multicast address
/// 
/// 
pub async fn send_messages(client: &UdpSocket, num_messages: u32) -> std::io::Result<()> {
    let ip = network_multicast_ip();
    let port = multicast_port();
    let location = format!("{ip}:{port}");
    
    for _ in 0..num_messages {
        let message = "Hello from client!";
        client.send_to(message.as_bytes(), &location).expect("Failed to send message");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await; // Add a 1-second delay
    }
    
    Ok(())
}

/// Start client server
/// 
/// 
pub async fn start_client() -> Result<(), Box<dyn Error>> {
    let ip = "0.0.0.0";
    
    // Create a test client and bind it to an ephemeral port
    let client = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");
    
    // Get the assigned port
    let client_port = client.local_addr().unwrap().port();
    
    let client = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
    client.set_multicast_loop_v4(true).expect("Failed to set multicast loop");
    let multicast_addr = Ipv4Addr::from_str(&network_multicast_ip())?;
    let interface_addr = Ipv4Addr::new(0, 0, 0, 0);
    client.join_multicast_v4(&multicast_addr, &interface_addr).expect("Failed to join multicast group");
    
    println!("Client UDP socket binded to: {ip}:{client_port}");
    println!("Sending to multicast address: {multicast_addr}");
    
    loop {
        send_messages(&client, 1).await?;
        println!("Message sent");
    }
}

/// Test server
/// 
/// 
#[tokio::test]
async fn test_server() {
    let ip = "0.0.0.0";
    
    // Create a test client and bind it to an ephemeral port
    let client = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");
    
    // Get the assigned port
    let client_port = client.local_addr().unwrap().port();
    println!("Client UDP socket binded to: {ip}:{client_port}");
    
    client.set_multicast_loop_v4(true).expect("Failed to set multicast loop");
    let multicast_addr = Ipv4Addr::from_str(&network_multicast_ip()).unwrap();
    let interface_addr = Ipv4Addr::new(0, 0, 0, 0);
    client.join_multicast_v4(&multicast_addr, &interface_addr).expect("Failed to join multicast group");
    
    // // Start the server in a separate task
    // tokio::spawn(async move {
    //     start_server().await.unwrap();
    // });
    
    // Send 5 messages to the server
    send_messages(&client, 5).await.unwrap();
    
    // Receive the response from the server
    let mut buffer = [0; 1024];
    let (bytes_received, _) = client.recv_from(&mut buffer).expect("Failed to receive message");
    
    // Assert that the response matches the expected message
    let expected_message = "Hello from client!";
    assert_eq!(&buffer[..bytes_received], expected_message.as_bytes());
}