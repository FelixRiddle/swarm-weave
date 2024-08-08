// use std::{error::Error, net::SocketAddr};
// use std::net::{
//     UdpSocket,
//     Ipv4Addr,
//     Ipv6Addr,
// };
// use socket2::SockAddr;

// use super::server::new_socket;

// /// New sender
// /// 
// /// 
// pub fn new_sender(addr: &SocketAddr) -> Result<UdpSocket, Box<dyn Error>> {
//     let socket = new_socket(addr)?;
    
//     if addr.is_ipv4() {
//         socket.set_multicast_if_v4(
//             &Ipv4Addr::new(0, 0, 0, 0)
//         )?;
        
//         socket.bind(&SockAddr::from(SocketAddr::new(
//             Ipv4Addr::new(0, 0, 0, 0).into(),
//             0
//         )))?;
//     } else {
//         socket.set_multicast_if_v6(5)?;
        
//         socket.bind(&SockAddr::from(SocketAddr::new(
//             Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(),
//             0
//         )))?;
//     }
    
//     Ok(UdpSocket::from(socket))
// }
