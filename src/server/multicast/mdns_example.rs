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

#[cfg(test)]
pub mod tests {
    use tokio::test;
    use std::net::{
        Ipv4Addr,
        Ipv6Addr,
    };
    use dns_parser::Class;
    
    use super::*;
    
    // Discover nodes can't be tested, because it yields this error:
    // `Cannot start a runtime from within a runtime. This happens because a function (like `block_on`)
    // attempted to block the current thread while the thread is being used to drive asynchronous tasks.`
    
    #[test]
    async fn test_to_ip_addr() {
        let record_a = Record {
            name: "example.local.".to_string(),
            class: Class::IN,
            ttl: 3600, // Time to live (1 hour)
            kind: RecordKind::A(Ipv4Addr::new(192, 168, 1, 100)),
        };
        let record_aaaa = Record {
            name: "example.local.".to_string(),
            class: Class::IN,
            ttl: 3600, // Time to live (1 hour)
            kind: RecordKind::AAAA(Ipv6Addr::new(
                0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334,
            )),
        };

        assert_eq!(to_ip_addr(&record_a), Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))));
        assert_eq!(to_ip_addr(&record_aaaa), Some(IpAddr::V6(Ipv6Addr::new(
            0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334,
        ))));
    }
}
