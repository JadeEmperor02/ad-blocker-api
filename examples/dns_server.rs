use ad_blocker_api::prelude::*;
use anyhow::Result;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use tokio::net::UdpSocket as TokioUdpSocket;

/// Simple DNS server using your ad blocker
#[tokio::main]
async fn main() -> Result<()> {
    println!("🛡️ Rust Ad Blocker DNS Server");
    println!("==============================");
    
    // Create ad blocker
    let blocker = Arc::new(SimpleAdBlocker::new().await?);
    
    let dns_port = 53;
    let addr: SocketAddr = format!("0.0.0.0:{}", dns_port).parse()?;
    
    println!("🌐 Starting DNS server on port {}...", dns_port);
    
    // Bind UDP socket
    let socket = TokioUdpSocket::bind(addr).await?;
    println!("✅ DNS server listening on {}", addr);
    println!("📱 Configure your devices to use this server's IP as DNS");
    println!("🔧 Press Ctrl+C to stop");
    println!();
    
    let mut query_count = 0u64;
    let mut blocked_count = 0u64;
    
    loop {
        let mut buffer = [0; 512];
        
        match socket.recv_from(&mut buffer).await {
            Ok((size, client_addr)) => {
                query_count += 1;
                
                // Parse DNS query (simplified)
                if let Some(domain) = extract_domain_from_dns_query(&buffer[..size]) {
                    println!("📱 Query #{}: {} from {}", query_count, domain, client_addr);
                    
                    // Check if should be blocked
                    match blocker.check_url(&format!("http://{}", domain)).await {
                        Ok(block_result) => {
                            if block_result.should_block {
                                blocked_count += 1;
                                println!("   🚫 BLOCKED: {}", block_result.reason);
                                
                                // Send blocked response (0.0.0.0)
                                let response = create_blocked_dns_response(&buffer[..size]);
                                let _ = socket.send_to(&response, client_addr).await;
                            } else {
                                println!("   ✅ ALLOWED: Forwarding to upstream DNS");
                                
                                // Forward to upstream DNS (8.8.8.8)
                                forward_dns_query(&socket, &buffer[..size], client_addr).await;
                            }
                        }
                        Err(e) => {
                            println!("   ❌ Error checking domain: {}", e);
                            // Forward on error
                            forward_dns_query(&socket, &buffer[..size], client_addr).await;
                        }
                    }
                    
                    // Show stats every 25 queries
                    if query_count % 25 == 0 {
                        let block_rate = (blocked_count as f64 / query_count as f64) * 100.0;
                        println!("📊 Stats: {}/{} queries blocked ({:.1}%)", 
                            blocked_count, query_count, block_rate);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving DNS query: {}", e);
            }
        }
    }
}

fn extract_domain_from_dns_query(data: &[u8]) -> Option<String> {
    if data.len() < 12 {
        return None; // Too short for DNS header
    }
    
    // Skip DNS header (12 bytes)
    let mut pos = 12;
    let mut domain = String::new();
    
    while pos < data.len() {
        let len = data[pos] as usize;
        if len == 0 {
            break; // End of domain name
        }
        
        pos += 1;
        if pos + len > data.len() {
            return None; // Invalid length
        }
        
        if !domain.is_empty() {
            domain.push('.');
        }
        
        // Extract label
        for i in 0..len {
            if pos + i < data.len() {
                domain.push(data[pos + i] as char);
            }
        }
        
        pos += len;
    }
    
    if domain.is_empty() {
        None
    } else {
        Some(domain)
    }
}

fn create_blocked_dns_response(query: &[u8]) -> Vec<u8> {
    if query.len() < 12 {
        return Vec::new();
    }
    
    let mut response = query.to_vec();
    
    // Set response flags (QR=1, AA=1, RA=1)
    response[2] = 0x81;
    response[3] = 0x80;
    
    // Set answer count to 1
    response[6] = 0x00;
    response[7] = 0x01;
    
    // Add answer section (points to 0.0.0.0)
    response.extend_from_slice(&[
        0xc0, 0x0c, // Name pointer to question
        0x00, 0x01, // Type A
        0x00, 0x01, // Class IN
        0x00, 0x00, 0x00, 0x3c, // TTL (60 seconds)
        0x00, 0x04, // Data length (4 bytes)
        0x00, 0x00, 0x00, 0x00, // IP address 0.0.0.0
    ]);
    
    response
}

async fn forward_dns_query(socket: &TokioUdpSocket, query: &[u8], client_addr: SocketAddr) {
    // Forward to Google DNS (8.8.8.8:53)
    let upstream_addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
    
    // Create a new socket for upstream query
    if let Ok(upstream_socket) = UdpSocket::bind("0.0.0.0:0") {
        if upstream_socket.send_to(query, upstream_addr).is_ok() {
            let mut buffer = [0; 512];
            if let Ok((size, _)) = upstream_socket.recv_from(&mut buffer) {
                // Forward response back to client
                let _ = socket.send_to(&buffer[..size], client_addr).await;
            }
        }
    }
}