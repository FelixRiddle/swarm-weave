use futures_util::{pin_mut, stream::StreamExt};
use mdns::{discover, Record, RecordKind};
use std::error::Error;
use std::net::IpAddr;
use std::time::Duration;

/// Discover nodes
/// 
/// 
pub async fn discover_nodes() -> Result<(), Box<dyn Error>> {
    let service_name = "_http._tcp.local";
    
    // Iterate through responses from each Cast device, asking for new devices every 15s
    let stream = discover::all(service_name, Duration::from_secs(15))?.listen();
    pin_mut!(stream);
    
    while let Some(Ok(response)) = stream.next().await {
        let addr = response.records()
            .filter_map(self::to_ip_addr)
            .next();
        
        if let Some(addr) = addr {
            println!("Found cast device at: {}", addr);
        } else {
            println!("Cast device doesn't advertise address");
        }
    }
    
    Ok(())
}

/// To ip address
/// 
/// 
pub fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => {
            Some(addr.into())
        }
        RecordKind::AAAA(addr) => {
            Some(addr.into())
        }
        _ => None,
    }
}
