use ad_blocker_api::prelude::*;
use anyhow::Result;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Local DNS server for testing (uses port 5353 to avoid needing root)
#[tokio::main]
async fn main() -> Result<()> {
    println!("🛡️  Local DNS Ad Blocker (Test Mode)");
    println!("=====================================");
    println!("Uses port 5353 (no root required)");
    println!();
    
    // Create StevenBlack blocker
    let blocker = Arc::new(StevenBlackBlocker::new().await?);
    
    // Get local IP
    let local_ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    let dns_port = 5354; // Non-privileged port
    
    println!("🌐 Starting local DNS server...");
    println!("📍 DNS Server: {}:{}", local_ip, dns_port);
    println!();
    
    println!("📱 PHONE SETUP (Test Mode):");
    println!("Since we're using port 5353, you'll need to:");
    println!("1. Use a DNS app that supports custom ports");
    println!("2. Or test with dig/nslookup from computer:");
    println!("   dig @{} -p {} google.com", local_ip, dns_port);
    println!("   dig @{} -p {} doubleclick.net", local_ip, dns_port);
    println!();
    
    println!("🚀 For REAL phone use:");
    println!("   Run: cargo run --example vpn_server (requires sudo)");
    println!("   Or deploy to cloud server (see VPN_DEPLOYMENT.md)");
    println!();
    
    // Bind UDP socket
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", dns_port))?;
    println!("✅ Local DNS server running on port {}!", dns_port);
    println!("🔍 Monitoring DNS queries...\n");
    
    let stats = Arc::new(RwLock::new(LocalDnsStats::default()));
    
    // Spawn stats reporter
    let stats_clone = stats.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let current_stats = stats_clone.read().await;
            
            if current_stats.total_queries > 0 {
                let block_rate = (current_stats.blocked_queries as f64 / current_stats.total_queries as f64) * 100.0;
                println!("📊 Stats: {}/{} queries blocked ({:.1}%)", 
                    current_stats.blocked_queries, 
                    current_stats.total_queries,
                    block_rate
                );
            }
        }
    });
    
    // Main DNS server loop
    loop {
        let mut buffer = [0; 512];
        let (size, client_addr) = socket.recv_from(&mut buffer)?;
        
        let query_data = buffer[..size].to_vec();
        let blocker_clone = blocker.clone();
        let stats_clone = stats.clone();
        let socket_clone = socket.try_clone()?;
        
        // Handle DNS query in background
        tokio::spawn(async move {
            match handle_dns_query(&query_data, client_addr, &blocker_clone, &stats_clone).await {
                Ok(response) => {
                    if let Err(e) = socket_clone.send_to(&response, client_addr) {
                        eprintln!("Error sending DNS response: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error handling DNS query: {}", e);
                }
            }
        });
    }
}

#[derive(Debug, Default)]
struct LocalDnsStats {
    total_queries: u64,
    blocked_queries: u64,
    forwarded_queries: u64,
}

async fn handle_dns_query(
    query_data: &[u8], 
    client_addr: SocketAddr, 
    blocker: &StevenBlackBlocker,
    stats: &Arc<RwLock<LocalDnsStats>>
) -> Result<Vec<u8>> {
    let mut current_stats = stats.write().await;
    current_stats.total_queries += 1;
    drop(current_stats);
    
    // Simple DNS parsing (just extract domain name)
    if let Some(domain) = parse_simple_dns_query(query_data) {
        println!("📱 DNS Query from {}: {}", client_addr.ip(), domain);
        
        // Check if domain should be blocked
        if blocker.is_blocked(&domain).await {
            let mut current_stats = stats.write().await;
            current_stats.blocked_queries += 1;
            drop(current_stats);
            
            println!("   🚫 BLOCKED: {}", domain);
            return Ok(create_blocked_dns_response(query_data));
        }
        
        println!("   ✅ ALLOWED: {}", domain);
    }
    
    // Forward to real DNS server
    let mut current_stats = stats.write().await;
    current_stats.forwarded_queries += 1;
    drop(current_stats);
    
    forward_to_upstream_dns(query_data).await
}

fn parse_simple_dns_query(data: &[u8]) -> Option<String> {
    if data.len() < 12 {
        return None;
    }
    
    // Skip DNS header (12 bytes)
    let mut pos = 12;
    let mut domain_parts = Vec::new();
    
    while pos < data.len() {
        let len = data[pos] as usize;
        if len == 0 {
            break;
        }
        
        pos += 1;
        if pos + len > data.len() {
            break;
        }
        
        let part = String::from_utf8_lossy(&data[pos..pos + len]);
        domain_parts.push(part.to_string());
        pos += len;
    }
    
    if domain_parts.is_empty() {
        None
    } else {
        Some(domain_parts.join("."))
    }
}

fn create_blocked_dns_response(query: &[u8]) -> Vec<u8> {
    let mut response = query.to_vec();
    
    // Set response flags (QR=1, AA=1, RA=1)
    if response.len() >= 3 {
        response[2] = 0x81;
        response[3] = 0x80;
    }
    
    // Add answer section pointing to 0.0.0.0 (blocked)
    response.extend_from_slice(&[
        0xc0, 0x0c, // Name pointer to query
        0x00, 0x01, // Type A
        0x00, 0x01, // Class IN
        0x00, 0x00, 0x00, 0x3c, // TTL (60 seconds)
        0x00, 0x04, // Data length
        0x00, 0x00, 0x00, 0x00, // IP address 0.0.0.0 (blocked)
    ]);
    
    // Update answer count
    if response.len() >= 8 {
        response[6] = 0x00;
        response[7] = 0x01;
    }
    
    response
}

async fn forward_to_upstream_dns(query_data: &[u8]) -> Result<Vec<u8>> {
    // Forward to Google DNS
    let upstream_socket = UdpSocket::bind("0.0.0.0:0")?;
    upstream_socket.connect("8.8.8.8:53")?;
    
    upstream_socket.send(query_data)?;
    
    let mut buffer = [0; 512];
    let size = upstream_socket.recv(&mut buffer)?;
    
    Ok(buffer[..size].to_vec())
}

fn get_local_ip() -> Option<String> {
    use std::net::TcpStream;
    
    if let Ok(stream) = TcpStream::connect("8.8.8.8:80") {
        if let Ok(local_addr) = stream.local_addr() {
            return Some(local_addr.ip().to_string());
        }
    }
    
    None
}