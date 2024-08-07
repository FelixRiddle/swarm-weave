// //! Trying to build a multicast server
// //! 
// //! Reference/s:
// //! https://bluejekyll.github.io/blog/posts/multicasting-in-rust/
// use socket2::{Domain, Protocol, SockAddr, Socket, Type};
// use std::error::Error;
// use std::mem::MaybeUninit;
// use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
// use std::slice;
// use std::sync::{Arc, Barrier};
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::thread::JoinHandle;
// use std::time::Duration;
// use std::io;

// use crate::config::env::multicast_port;
// use crate::multicast::socket_2::client::new_sender;

// /// On Windows, unlike all Unix variants, it is improper to bind to the multicast address
// ///
// /// see https://msdn.microsoft.com/en-us/library/windows/desktop/ms737550(v=vs.85).aspx
// #[cfg(windows)]
// pub fn bind_multicast(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
//     let addr = match *addr {
//         SocketAddr::V4(addr) => SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), addr.port()),
//         SocketAddr::V6(addr) => SocketAddr::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), addr.port())
//     };
    
//     socket.bind(&SockAddr::from(addr))
// }

// /// On unixes we bind to the multicast address, which causes multicast packets to be filtered
// /// 
// /// 
// #[cfg(unix)]
// pub fn bind_multicast(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
//     socket.bind(&SockAddr::from(*addr))
// }

// /// This will be common for all sockets
// /// 
// /// 
// pub fn new_socket(addr: &SocketAddr) -> Result<Socket, Box<dyn Error>> {
//     let domain = if addr.is_ipv4() {
//         Domain::IPV4
//     } else {
//         Domain::IPV6
//     };
    
//     let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP)).map_err(|e| {
//         Box::new(e) // Wrap the error in a Box<dyn Error>
//     })?;
    
//     // We're going to use read timeouts so that we don't hang waiting for packets
//     socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    
//     Ok(socket)
// }

// /// Multicast listener
// /// 
// /// 
// // pub fn multicast_listener(
// //     response: &'static str,
// //     client_done: Arc<AtomicBool>,
// //     addr: SocketAddr,
// // ) -> Result<JoinHandle<()>, Box<dyn Error>> {
// //     // A barrier to not start the client test code until after the server is running
// //     let server_barrier = Arc::new(Barrier::new(2));
// //     let client_barrier = Arc::clone(&server_barrier);
    
// //     let join_handle = std::thread::Builder::new()
// //         .name(format!("{}:server", response))
// //         .spawn(move || {
// //             // Socket creation will go here...
// //             let listener = join_multicast(addr).expect("Failed to create listener");
// //             println!("{}:server: Joined: {}", response, addr);
            
// //             server_barrier.wait();
// //             println!("{}:server Is ready", response);
            
// //             // We'll be looping until the client indicates it is done.
// //             while !client_done.load(std::sync::atomic::Ordering::Relaxed) {
// //                 // Test receive and response code will go here...
// //                 // Receive buffer
// //                 let mut buf: [MaybeUninit<u8>; 2] = [MaybeUninit::new(0), MaybeUninit::new(64)];
// //                 let mut data = unsafe {
// //                     slice::from_raw_parts_mut(
// //                         buf.as_mut_ptr() as *mut u8,
// //                         64
// //                     )
// //                 };
                
// //                 // We're assuming failures were timeouts, the client_done loop will stop us
// //                 match listener.recv_from(&mut buf) {
// //                     Ok((len, remote_addr)) => {
// //                         let data = &data[..len];
                        
// //                         println!(
// //                             "{}:server: Got data: {} from: {:?}",
// //                             response,
// //                             String::from_utf8_lossy(data),
// //                             remote_addr
// //                         );
                        
// //                         let remote_addr: SocketAddr = convert_socket_addr(remote_addr);
                        
// //                         // Create a socket to send the response
// //                         let responder = new_socket(
// //                                 &remote_addr
// //                             ).expect("Failed to create responder")
// //                             .into_udp_socket();
                        
// //                         // We send the response that was set at the method beginning
// //                         responder
// //                             .send_to(response.as_bytes(), &remote_addr)
// //                             .expect("Failed to respond");
                        
// //                         println!("{}:server: Sent response to: {}", response, remote_addr);
// //                     }
// //                     Err(err) => {
// //                         println!("{}:server: Got an error {}", response, err);
// //                     }
// //                 }
// //             }
            
// //             println!("{}:server: Client is done", response);
// //         })?;
    
// //     client_barrier.wait();
    
// //     let listener = join_multicast(addr)?;
    
// //     Ok(join_handle)
// // }

// /// This will guarantee we always tell the server to stop
// /// 
// struct NotifyServer(Arc<AtomicBool>);
// impl Drop for NotifyServer {
//     fn drop(&mut self) {
//         self.0.store(true, Ordering::Relaxed)
//     }
// }

// /// Our generic test over different IPs
// /// 
// /// 
// fn test_multicast(test: &'static str, addr: IpAddr) {
//     assert!(addr.is_multicast());
//     let addr = SocketAddr::new(addr, multicast_port().parse::<u16>().unwrap());
    
//     let client_done = Arc::new(AtomicBool::new(false));
//     let notify = NotifyServer(Arc::clone(&client_done));
    
//     // multicast_listener(test, client_done, addr).unwrap();
    
//     let message = b"Hello from client!";
    
//     let socket = new_sender(&addr)
//         .expect("Couldn't create sender!");
    
//     socket.send_to(message, &addr.to_socket_addrs().expect("Couldn't convert to SocketAddr").next().expect("No address found"))
//         .expect("Couldn't send_to!");
    
//     let mut buf = [0u8, 64];
    
//     // Get our expected response
//     match socket.recv_from(&mut buf) {
//         Ok((len, _remote_addr)) => {
//             let data = &buf[..len];
//             let response = String::from_utf8_lossy(data);
            
//             println!("{}:client: Got data: {}", test, response);
            
//             // Verify it's what we expected
//             assert_eq!(test, response);
//         }
//         Err(err) => {
//             println!("{}:client: Had a problem: {}", test, err);
//             assert!(false);
//         }
//     };
    
//     // Make sure we don't notify the server until the end of the client test
//     drop(notify);
// }

// /// Join multicast
// /// 
// /// 
// pub fn join_multicast(addr: SocketAddr) -> Result<Socket, Box<dyn Error>> {
//     let ip_addr = match addr {
//         SocketAddr::V4(v4) => {
//             let ip = v4.ip().clone();
//             Some(ip)
//         }
//         SocketAddr::V6(_) => None,
//     };
    
//     let socket = new_socket(&addr)?;
    
//     // Depending on the IP protocol we have slightly different work
//     let result = match ip_addr {
//         Some(ref mdns_v4) => {
//             // Join to the multicast address, with all interfaces
//             socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
            
//             // Bind us to the socket address
//             socket.bind(&SockAddr::from(addr))?;
            
//             Ok(socket)
//         }
//         None => {
//             Err("Not a valid IPv4 address".into())
//         }
//     };
    
//     result
// }
