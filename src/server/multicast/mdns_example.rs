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
    use super::*;
    use serde_json::json;
    use tokio::test;
    use mockito::Server;
    use std::net::{
        Ipv4Addr,
        Ipv6Addr,
    };
    use dns_parser::Class;
    
    #[test]
    async fn test_discover_nodes() {
        // Mock mDNS responses
        let mut server = Server::new();
        
        let body = json!([
            {
                "name": "device1",
                "records": [
                    {"kind": "A", "addr": "192.168.1.100"}
                ]
            },
            {
                "name": "device2",
                "records": [
                    {"kind": "AAAA", "addr": "2001:0db8:85a3:0000:0000:8a2e:0370:7334"}
                ]
            }
        ]);
        let body = serde_json::to_string(&body).unwrap();
        
        let _mock_response = server.mock("GET", "/_http._tcp.local.")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(body)
            .create();

        // Run discover_nodes with mocked responses
        let result = discover_nodes().await;
        assert!(result.is_ok());

        // Verify discovered nodes
        let expected_nodes = vec![
            "192.168.1.100",
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
        ];
        let discovered_nodes: Vec<String> = std::io::stdin()
            .lines()
            .map(|line| line.unwrap())
            .filter(|line| line.starts_with("Found cast device at:"))
            .map(|line| line.trim_start_matches("Found cast device at:").to_string())
            .map(|line| line.to_string()) // Convert &str to String
            .collect();
        assert_eq!(discovered_nodes, expected_nodes);
    }

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
