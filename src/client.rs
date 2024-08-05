use std::net::{UdpSocket, Ipv4Addr};
use std::time::Duration;
use tokio::time::sleep;

/// Start client server
/// 
/// 
pub async fn start_client() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8081").expect("Failed to bind socket");
    socket.set_multicast_loop_v4(true).expect("Failed to set multicast loop");
    let multicast_addr = Ipv4Addr::new(239, 1, 1, 1);
    let interface_addr = Ipv4Addr::new(0, 0, 0, 0);
    socket.join_multicast_v4(&multicast_addr, &interface_addr).expect("Failed to join multicast group");
    
    loop {
        let message = "Hello from client!";
        socket.send_to(message.as_bytes(), "239.1.1.1:8080").expect("Failed to send message");
        sleep(Duration::from_secs(1)).await; // Add a 1-second delay
    }
}

#[tokio::test]
async fn test_server() {
    let ip = "0.0.0.0";
    
    // Create a test client and bind it to an ephemeral port
    let client = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");

    // Get the assigned port
    let client_port = client.local_addr().unwrap().port();
    println!("Client UDP socket binded to: {ip}:{client_port}");
    
    // Send a message to the server
    let message = "Hello from client!";
    client.send_to(message.as_bytes(), "239.1.1.1:8080").expect("Failed to send message");

    // Receive the response from the server
    let mut buffer = [0; 1024];
    let (bytes_received, _) = client.recv_from(&mut buffer).expect("Failed to receive message");

    // Assert that the response matches the expected message
    let expected_message = "Hello from client!";
    assert_eq!(&buffer[..bytes_received], expected_message.as_bytes());
}
