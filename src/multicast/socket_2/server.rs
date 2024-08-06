//! Trying to build a multicast server
//! 
//! Reference/s:
//! https://bluejekyll.github.io/blog/posts/multicasting-in-rust/
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Barrier};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::config::env::multicast_port;

/// Multicast listener
/// 
/// 
pub fn multicast_listener(
    response: &'static str,
    client_done: Arc<AtomicBool>,
    addr: SocketAddr,
) -> Result<JoinHandle<()>, Box<dyn Error>> {
    // A barrier to not start the client test code until after the server is running
    let server_barrier = Arc::new(Barrier::new(2));
    let client_barrier = Arc::clone(&server_barrier);
    
    let join_handle = std::thread::Builder::new()
        .name(format!("{}:server", response))
        .spawn(move || {
            // socket creation will go here...
            
            server_barrier.wait();
            println!("{}:server Is ready", response);
            
            // We'll be looping until the client indicates it is done.
            while !client_done.load(std::sync::atomic::Ordering::Relaxed) {
                // test receive and response code will go here...
            }
            
            println!("{}:server: Client is done", response);
        })?;
    
    client_barrier.wait();
    
    let listener = join_multicast(addr)?;
    
    Ok(join_handle)
}

/// This will guarantee we always tell the server to stop
/// 
struct NotifyServer(Arc<AtomicBool>);
impl Drop for NotifyServer {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed)
    }
}

/// Our generic test over different IPs
/// 
/// 
fn test_multicast(test: &'static str, addr: IpAddr) {
    assert!(addr.is_multicast());
    let addr = SocketAddr::new(addr, multicast_port().parse::<u16>().unwrap());
    
    let client_done = Arc::new(AtomicBool::new(false));
    NotifyServer(Arc::clone(&client_done));
    
    multicast_listener(test, client_done, addr).unwrap();
    
    // Client test code send and receive code after here
    println!("{}:client Running", test)
}

/// This will be common for all sockets
/// 
/// 
pub fn new_socket(addr: &SocketAddr) -> Result<Socket, Box<dyn Error>> {
    let domain = if addr.is_ipv4() {
        Domain::IPV4
    } else {
        Domain::IPV6
    };
    
    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;
    
    // We're going to use read timeouts so that we don't hang waiting for packets
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    
    Ok(socket)
}

/// Join multicast
/// 
/// 
pub fn join_multicast(addr: SocketAddr) -> Result<Socket, Box<dyn Error>> {
    let ip_addr = match addr {
        SocketAddr::V4(v4) => {
            let ip = v4.ip().clone();
            Some(ip)
        }
        SocketAddr::V6(_) => None,
    };
    
    let socket = new_socket(&addr)?;
    
    // Depending on the IP protocol we have slightly different work
    let result = match ip_addr {
        Some(ref mdns_v4) => {
            // Join to the multicast address, with all interfaces
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
            
            // Bind us to the socket address
            socket.bind(&SockAddr::from(addr))?;
            
            Ok(socket)
        }
        None => {
            Err("Not a valid IPv4 address".into())
        }
    };
    
    result
}
