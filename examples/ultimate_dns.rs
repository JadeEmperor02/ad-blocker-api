use ad_blocker_api::prelude::*;
use anyhow::Result;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;

/// Ultimate DNS server with 1M+ blocked domains
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Ultimate Ad Blocker DNS Server");
    println!("=================================");
    println!("📥 Loading massive blocklist...");
    
    // Load the massive blocklist
    let blocked_domains = load_blocklist("/tmp/clean_blocklist.txt").await?;
    let blocked_count = blocked_domains.len();
    
    println!("✅ Loaded {} blocked domains!", blocked_count);
    
    // Also create the regular ad blocker for additional filtering
    let blocker = Arc::new(SimpleAdBlocker::new().await?);
    let blocked_domains = Arc::new(blocked_domains);
    
    let dns_port = 53;
    let addr: SocketAddr = format!("0.0.0.0:{}", dns_port).parse()?;
    
    println!("🌐 Starting ultimate DNS server on port {}...", dns_port);
    
    let socket = UdpSocket::bind(addr)?;
    println!("✅ Ultimate DNS server listening on {}", addr);
    println!("📱 Configure your devices to use this server's IP as DNS");
    println!("🛡️ Blocking {:.1}M+ domains!", blocked_count as f64 / 1_000_000.0);
    println!("🔧 Press Ctrl+C to stop");
    println!();
    
    let mut query_count = 0u64;
    let mut blocked_count = 0u64;
    
    loop {
        let mut buffer = [0; 512];
        
        match socket.recv_from(&mut buffer) {
            Ok((size, client_addr)) => {
                query_count += 1;
                
                if let Some(domain) = extract_domain_from_dns_query(&buffer[..size]) {
                    println!("📱 Query #{}: {} from {}", query_count, domain, client_addr);
                    
                    let should_block = check_domain_blocked(&domain, &blocked_domains, &blocker).await;
                    
                    if should_block {
                        blocked_count += 1;
                        println!("   🚫 BLOCKED: Domain in ultimate blocklist");
                        
                        let response = create_blocked_dns_response(&buffer[..size]);
                        let _ = socket.send_to(&response, client_addr);
                    } else {
                        println!("   ✅ ALLOWED: Forwarding to upstream DNS");
                        forward_dns_query(&socket, &buffer[..size], client_addr);
                    }
                    
                    // Show stats every 25 queries
                    if query_count % 25 == 0 {
                        let block_rate = (blocked_count as f64 / query_count as f64) * 100.0;
                        println!("📊 Ultimate Stats: {}/{} queries blocked ({:.1}%) | {:.1}M domains loaded", 
                            blocked_count, query_count, block_rate, blocked_domains.len() as f64 / 1_000_000.0);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving DNS query: {}", e);
            }
        }
    }
}

async fn load_blocklist(file_path: &str) -> Result<HashSet<String>> {
    let mut domains = HashSet::new();
    
    match File::open(file_path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(domain) = line {
                    let domain = domain.trim().to_lowercase();
                    if !domain.is_empty() && !domain.starts_with('#') {
                        domains.insert(domain);
                    }
                }
            }
            println!("📂 Loaded {} domains from {}", domains.len(), file_path);
        }
        Err(_) => {
            println!("⚠️  Blocklist file not found, using built-in filtering only");
        }
    }
    
    Ok(domains)
}

async fn check_domain_blocked(
    domain: &str, 
    blocked_domains: &HashSet<String>, 
    blocker: &SimpleAdBlocker
) -> bool {
    // Check against our massive blocklist first (fastest)
    if blocked_domains.contains(domain) {
        return true;
    }
    
    // Check subdomains (e.g., if "ads.example.com" and we have "example.com")
    let parts: Vec<&str> = domain.split('.').collect();
    for i in 1..parts.len() {
        let subdomain = parts[i..].join(".");
        if blocked_domains.contains(&subdomain) {
            return true;
        }
    }
    
    // Fallback to the advanced ad blocker
    match blocker.check_url(&format!("http://{}", domain)).await {
        Ok(result) => result.should_block,
        Err(_) => false,
    }
}

fn extract_domain_from_dns_query(data: &[u8]) -> Option<String> {
    if data.len() < 12 {
        return None;
    }
    
    let mut pos = 12;
    let mut domain = String::new();
    
    while pos < data.len() {
        let len = data[pos] as usize;
        if len == 0 {
            break;
        }
        
        pos += 1;
        if pos + len > data.len() {
            return None;
        }
        
        if !domain.is_empty() {
            domain.push('.');
        }
        
        for i in 0..len {
            if pos + i < data.len() && data[pos + i].is_ascii() {
                domain.push(data[pos + i] as char);
            }
        }
        
        pos += len;
    }
    
    if domain.is_empty() {
        None
    } else {
        Some(domain.to_lowercase())
    }
}

fn create_blocked_dns_response(query: &[u8]) -> Vec<u8> {
    if query.len() < 12 {
        return Vec::new();
    }
    
    let mut response = query.to_vec();
    
    response[2] = 0x81;
    response[3] = 0x80;
    response[6] = 0x00;
    response[7] = 0x01;
    
    response.extend_from_slice(&[
        0xc0, 0x0c,
        0x00, 0x01,
        0x00, 0x01,
        0x00, 0x00, 0x00, 0x3c,
        0x00, 0x04,
        0x00, 0x00, 0x00, 0x00,
    ]);
    
    response
}

fn forward_dns_query(socket: &UdpSocket, query: &[u8], client_addr: SocketAddr) {
    let upstream_addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
    
    if let Ok(upstream_socket) = UdpSocket::bind("0.0.0.0:0") {
        if upstream_socket.send_to(query, upstream_addr).is_ok() {
            let mut buffer = [0; 512];
            if let Ok((size, _)) = upstream_socket.recv_from(&mut buffer) {
                let _ = socket.send_to(&buffer[..size], client_addr);
            }
        }
    }
}