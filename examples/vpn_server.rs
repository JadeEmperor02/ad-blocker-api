use ad_blocker_api::prelude::*;
use anyhow::Result;
use hickory_client::client::{Client, SyncClient};
use hickory_client::udp::UdpClientConnection;
use hickory_proto::op::{DnsResponse, Message, OpCode, Query};
use hickory_proto::rr::{DNSClass, Name, RData, Record, RecordType};
use serde_json::json;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// VPN-style DNS server that blocks ads at the DNS level
/// This works on any network connection (WiFi, cellular, etc.)
#[derive(Clone)]
pub struct VpnAdBlocker {
    blocker: Arc<StevenBlackBlocker>,
    stats: Arc<RwLock<VpnStats>>,
    upstream_dns: SocketAddr,
}

#[derive(Debug, Default, Clone)]
pub struct VpnStats {
    pub total_queries: u64,
    pub blocked_queries: u64,
    pub forwarded_queries: u64,
    pub unique_blocked_domains: std::collections::HashSet<String>,
}

impl VpnAdBlocker {
    pub async fn new() -> Result<Self> {
        println!("🔄 Initializing VPN-style ad blocker...");
        let blocker = Arc::new(StevenBlackBlocker::new().await?);
        
        // Load comprehensive blocklists
        let additional_hosts = vec![
            "https://raw.githubusercontent.com/StevenBlack/hosts/master/alternates/fakenews-gambling/hosts",
            "https://someonewhocares.org/hosts/zero/hosts",
            "https://raw.githubusercontent.com/AdguardTeam/AdguardFilters/master/MobileFilter/sections/adservers.txt",
        ];
        
        println!("📥 Loading comprehensive blocklists...");
        if let Err(e) = blocker.load_additional_hosts(additional_hosts).await {
            eprintln!("⚠️  Warning: Could not load some additional hosts: {}", e);
        }
        
        Ok(Self {
            blocker,
            stats: Arc::new(RwLock::new(VpnStats::default())),
            upstream_dns: "8.8.8.8:53".parse()?,
        })
    }
    
    pub async fn handle_dns_query(&self, query_data: &[u8], client_addr: SocketAddr) -> Result<Vec<u8>> {
        let mut stats = self.stats.write().await;
        stats.total_queries += 1;
        drop(stats);
        
        // Parse DNS query
        let message = Message::from_vec(query_data)?;
        let query = message.queries().first().ok_or_else(|| anyhow::anyhow!("No query found"))?;
        let domain = query.name().to_string();
        
        // Remove trailing dot
        let clean_domain = domain.trim_end_matches('.');
        
        println!("📱 DNS Query from {}: {}", client_addr.ip(), clean_domain);
        
        // Check if domain should be blocked
        if self.blocker.is_blocked(clean_domain).await {
            let mut stats = self.stats.write().await;
            stats.blocked_queries += 1;
            stats.unique_blocked_domains.insert(clean_domain.to_string());
            drop(stats);
            
            println!("   🚫 BLOCKED: {}", clean_domain);
            return Ok(self.create_blocked_response(&message));
        }
        
        println!("   ✅ ALLOWED: Forwarding to upstream DNS");
        
        // Forward to upstream DNS
        match self.forward_dns_query(query_data).await {
            Ok(response) => {
                let mut stats = self.stats.write().await;
                stats.forwarded_queries += 1;
                Ok(response)
            }
            Err(e) => {
                eprintln!("   ❌ Error forwarding DNS query: {}", e);
                Ok(self.create_error_response(&message))
            }
        }
    }
    
    async fn forward_dns_query(&self, query_data: &[u8]) -> Result<Vec<u8>> {
        // Create UDP socket for upstream DNS
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(self.upstream_dns)?;
        
        // Send query to upstream DNS
        socket.send(query_data)?;
        
        // Receive response
        let mut buffer = [0; 512];
        let size = socket.recv(&mut buffer)?;
        
        Ok(buffer[..size].to_vec())
    }
    
    fn create_blocked_response(&self, original_message: &Message) -> Vec<u8> {
        let mut response = Message::new();
        response.set_id(original_message.id());
        response.set_message_type(hickory_proto::op::MessageType::Response);
        response.set_op_code(OpCode::Query);
        response.set_authoritative(true);
        response.set_recursion_desired(original_message.recursion_desired());
        response.set_recursion_available(true);
        
        // Copy the query
        for query in original_message.queries() {
            response.add_query(query.clone());
        }
        
        // Add answer pointing to 0.0.0.0 (blocked)
        if let Some(query) = original_message.queries().first() {
            if query.query_type() == RecordType::A {
                let mut record = Record::new();
                record.set_name(query.name().clone());
                record.set_record_type(RecordType::A);
                record.set_dns_class(DNSClass::IN);
                record.set_ttl(60); // 1 minute TTL
                record.set_data(Some(RData::A(Ipv4Addr::new(0, 0, 0, 0))));
                
                response.add_answer(record);
            }
        }
        
        response.to_vec().unwrap_or_else(|_| vec![])
    }
    
    fn create_error_response(&self, original_message: &Message) -> Vec<u8> {
        let mut response = Message::new();
        response.set_id(original_message.id());
        response.set_message_type(hickory_proto::op::MessageType::Response);
        response.set_op_code(OpCode::Query);
        response.set_response_code(hickory_proto::op::ResponseCode::ServFail);
        
        for query in original_message.queries() {
            response.add_query(query.clone());
        }
        
        response.to_vec().unwrap_or_else(|_| vec![])
    }
    
    pub async fn get_stats(&self) -> VpnStats {
        self.stats.read().await.clone()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🛡️  VPN-Style Ad Blocker DNS Server");
    println!("====================================");
    println!("Works on ANY network: WiFi, Cellular, Ethernet!");
    println!();
    
    // Create VPN ad blocker
    let vpn_blocker = VpnAdBlocker::new().await?;
    
    // Get local and public IP
    let local_ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    let public_ip = get_public_ip().await.unwrap_or_else(|| "your-server-ip".to_string());
    let dns_port = 53;
    
    println!("🌐 Starting VPN-style DNS server...");
    println!("📍 Local DNS server: {}:{}", local_ip, dns_port);
    println!("📍 Public DNS server: {}:{}", public_ip, dns_port);
    println!();
    
    // Show setup instructions
    show_setup_instructions(&local_ip, &public_ip, dns_port);
    
    // Bind UDP socket for DNS
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", dns_port))?;
    println!("✅ VPN DNS server running! Press Ctrl+C to stop.\n");
    
    // Spawn stats reporter
    let stats_blocker = vpn_blocker.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let stats = stats_blocker.get_stats().await;
            
            if stats.total_queries > 0 {
                let block_rate = (stats.blocked_queries as f64 / stats.total_queries as f64) * 100.0;
                println!("📊 VPN Stats: {}/{} queries blocked ({:.1}%), {} unique domains blocked", 
                    stats.blocked_queries, 
                    stats.total_queries,
                    block_rate,
                    stats.unique_blocked_domains.len()
                );
            }
        }
    });
    
    // Main DNS server loop
    loop {
        let mut buffer = [0; 512];
        let (size, client_addr) = socket.recv_from(&mut buffer)?;
        
        let blocker = vpn_blocker.clone();
        let query_data = buffer[..size].to_vec();
        
        // Handle DNS query
        tokio::spawn(async move {
            match blocker.handle_dns_query(&query_data, client_addr).await {
                Ok(response) => {
                    // Send response back to client
                    if let Err(e) = socket.send_to(&response, client_addr) {
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

fn show_setup_instructions(local_ip: &str, public_ip: &str, port: u16) {
    println!("📱 PHONE SETUP (Works on ANY network!):");
    println!();
    println!("🏠 LOCAL NETWORK (WiFi at home):");
    println!("   DNS Server: {}", local_ip);
    println!();
    println!("🌍 ANYWHERE (Cellular, other WiFi):");
    println!("   DNS Server: {}", public_ip);
    println!("   (You'll need to deploy this to a VPS/cloud server)");
    println!();
    
    println!("📱 iPhone Setup:");
    println!("   1. Settings → WiFi → (i) next to network");
    println!("   2. Configure DNS → Manual");
    println!("   3. Add DNS Server: {}", local_ip);
    println!("   4. Save");
    println!();
    
    println!("📱 Android Setup:");
    println!("   1. Settings → WiFi → Long press network");
    println!("   2. Modify Network → Advanced");
    println!("   3. IP Settings → Static");
    println!("   4. DNS 1: {}", local_ip);
    println!("   5. DNS 2: 8.8.8.8 (backup)");
    println!("   6. Save");
    println!();
    
    println!("🌐 For GLOBAL access (works anywhere):");
    println!("   1. Deploy this server to a VPS (DigitalOcean, AWS, etc.)");
    println!("   2. Use the VPS IP as your DNS server");
    println!("   3. Now it works on cellular, any WiFi, anywhere!");
    println!();
    
    println!("💡 Pro Tip: Use a service like Cloudflare or AWS to get a static IP");
    println!("🔒 Security: Consider adding authentication for public deployment");
    println!();
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

async fn get_public_ip() -> Option<String> {
    // Try to get public IP from external service
    if let Ok(response) = reqwest::get("https://api.ipify.org").await {
        if let Ok(ip) = response.text().await {
            return Some(ip.trim().to_string());
        }
    }
    
    None
}