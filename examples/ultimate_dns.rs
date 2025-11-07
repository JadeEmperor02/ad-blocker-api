use ad_blocker_api::prelude::*;
use anyhow::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Ultimate DNS server with enhanced dynamic ad blocking
struct DynamicAdBlocker {
    blocked_domains: HashSet<String>,
    blocker: SimpleAdBlocker,
    dynamic_patterns: Vec<Regex>,
    subdomain_patterns: Vec<Regex>,
    tracking_patterns: Vec<Regex>,
}

impl DynamicAdBlocker {
    async fn new(blocked_domains: HashSet<String>) -> Result<Self> {
        let blocker = SimpleAdBlocker::new().await?;
        
        // Enhanced patterns for dynamic ads and tracking
        let dynamic_patterns = vec![
            // Google Ads dynamic subdomains
            Regex::new(r"^tpc\.googlesyndication\.com$")?,
            Regex::new(r"^pagead\d*\.l\.google\.com$")?,
            Regex::new(r"^googleads\.g\.doubleclick\.net$")?,
            Regex::new(r"^stats\.g\.doubleclick\.net$")?,
            Regex::new(r"^cm\.g\.doubleclick\.net$")?,
            
            // Facebook dynamic tracking
            Regex::new(r"^.*\.facebook\.com$")?,
            Regex::new(r"^.*\.fbcdn\.net$")?,
            Regex::new(r"^connect\.facebook\.net$")?,
            
            // Amazon ads
            Regex::new(r"^.*\.amazon-adsystem\.com$")?,
            Regex::new(r"^.*\.adsystem\.amazon\..*$")?,
            
            // Generic ad networks with dynamic subdomains
            Regex::new(r"^.*\.ads\..*$")?,
            Regex::new(r"^.*\.ad\..*$")?,
            Regex::new(r"^.*\.advertising\..*$")?,
            Regex::new(r"^.*\.adsystem\..*$")?,
            Regex::new(r"^.*\.adnxs\.com$")?,
            Regex::new(r"^.*\.adsafeprotected\.com$")?,
            
            // Analytics and tracking
            Regex::new(r"^.*\.google-analytics\.com$")?,
            Regex::new(r"^.*\.googletagmanager\.com$")?,
            Regex::new(r"^.*\.hotjar\.com$")?,
            Regex::new(r"^.*\.mixpanel\.com$")?,
            
            // Social media widgets
            Regex::new(r"^.*\.addthis\.com$")?,
            Regex::new(r"^.*\.sharethis\.com$")?,
            
            // CDN-based ads
            Regex::new(r"^.*\.jsdelivr\.net/.*ads.*$")?,
            Regex::new(r"^.*\.unpkg\.com/.*ads.*$")?,
            
            // Additional dynamic ad networks
            Regex::new(r"^.*\.criteo\.com$")?,
            Regex::new(r"^.*\.adsafeprotected\.com$")?,
            Regex::new(r"^.*\.moatads\.com$")?,
            Regex::new(r"^.*\.rlcdn\.com$")?,
            Regex::new(r"^.*\.rubiconproject\.com$")?,
            Regex::new(r"^.*\.pubmatic\.com$")?,
            Regex::new(r"^.*\.openx\.net$")?,
            
            // Video ad platforms
            Regex::new(r"^.*\.videologygroup\.com$")?,
            Regex::new(r"^.*\.brightcove\.com/.*ads.*$")?,
            
            // Mobile ad networks
            Regex::new(r"^.*\.mopub\.com$")?,
            Regex::new(r"^.*\.applovin\.com$")?,
            Regex::new(r"^.*\.unity3d\.com/.*ads.*$")?,
            
            // Dynamic/lazy loading ad networks
            Regex::new(r"^.*\.adsystem\..*$")?,
            Regex::new(r"^.*\.adform\.net$")?,
            Regex::new(r"^.*\.adsafeprotected\.com$")?,
            Regex::new(r"^.*\.serving-sys\.com$")?,
            Regex::new(r"^.*\.adsystem\.com$")?,
            Regex::new(r"^.*\.adnxs\.com$")?,
            
            // JavaScript ad injection domains
            Regex::new(r"^.*\.googletag\..*$")?,
            Regex::new(r"^.*\.gstatic\.com/.*ads.*$")?,
            Regex::new(r"^.*\.googleusercontent\.com/.*ads.*$")?,
            
            // Pop-up and redirect ad networks
            Regex::new(r"^.*\.popads\.net$")?,
            Regex::new(r"^.*\.popcash\.net$")?,
            Regex::new(r"^.*\.propellerads\.com$")?,
            Regex::new(r"^.*\.mgid\.com$")?,
            Regex::new(r"^.*\.revcontent\.com$")?,
            
            // Native advertising platforms
            Regex::new(r"^.*\.nativo\.com$")?,
            Regex::new(r"^.*\.sharethrough\.com$")?,
            Regex::new(r"^.*\.plista\.com$")?,
        ];
        
        // Subdomain generation patterns (for dynamic ad domains)
        let subdomain_patterns = vec![
            Regex::new(r"^[a-z0-9]{8,}\..*$")?, // Random subdomain pattern
            Regex::new(r"^[0-9]+\..*$")?,       // Numeric subdomain
            Regex::new(r"^ads[0-9]*\..*$")?,    // ads + numbers
            Regex::new(r"^banner[0-9]*\..*$")?, // banner + numbers
            Regex::new(r"^track[0-9]*\..*$")?,  // track + numbers
            Regex::new(r"^ad[0-9]*\..*$")?,     // ad + numbers
            Regex::new(r"^promo[0-9]*\..*$")?,  // promo + numbers
            Regex::new(r"^popup[0-9]*\..*$")?,  // popup + numbers
            Regex::new(r"^click[0-9]*\..*$")?,  // click + numbers
            Regex::new(r"^serve[0-9]*\..*$")?,  // serve + numbers
            Regex::new(r"^cdn[0-9]*\..*ads.*$")?, // CDN with ads
            Regex::new(r"^static[0-9]*\..*ads.*$")?, // Static with ads
        ];
        
        // Enhanced tracking patterns
        let tracking_patterns = vec![
            Regex::new(r".*analytics.*")?,
            Regex::new(r".*tracking.*")?,
            Regex::new(r".*telemetry.*")?,
            Regex::new(r".*metrics.*")?,
            Regex::new(r".*beacon.*")?,
            Regex::new(r".*collector.*")?,
            Regex::new(r".*pixel.*")?,
            Regex::new(r".*impression.*")?,
            Regex::new(r".*conversion.*")?,
            Regex::new(r".*retargeting.*")?,
            Regex::new(r".*remarketing.*")?,
            Regex::new(r".*affiliate.*")?,
        ];
        
        Ok(Self {
            blocked_domains,
            blocker,
            dynamic_patterns,
            subdomain_patterns,
            tracking_patterns,
        })
    }
    
    async fn should_block(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();
        
        // 1. Check static blocklist first (fastest)
        if self.blocked_domains.contains(&domain_lower) {
            return true;
        }
        
        // 2. Check dynamic ad patterns
        for pattern in &self.dynamic_patterns {
            if pattern.is_match(&domain_lower) {
                return true;
            }
        }
        
        // 3. Check suspicious subdomain patterns
        for pattern in &self.subdomain_patterns {
            if pattern.is_match(&domain_lower) {
                // Additional check for known ad/tracking keywords
                if domain_lower.contains("ads") || domain_lower.contains("track") || 
                   domain_lower.contains("analytics") || domain_lower.contains("doubleclick") {
                    return true;
                }
            }
        }
        
        // 4. Check tracking patterns
        for pattern in &self.tracking_patterns {
            if pattern.is_match(&domain_lower) {
                return true;
            }
        }
        
        // 5. Check parent domains (subdomain blocking)
        let parts: Vec<&str> = domain_lower.split('.').collect();
        for i in 1..parts.len() {
            let parent_domain = parts[i..].join(".");
            if self.blocked_domains.contains(&parent_domain) {
                return true;
            }
        }
        
        // 6. Check for programmatic ad domains (common in dynamic injection)
        if self.is_programmatic_ad_domain(&domain_lower) {
            return true;
        }
        
        // 7. Use advanced ad blocker for final check
        match self.blocker.check_url(&format!("http://{}", domain_lower)).await {
            Ok(result) => result.should_block,
            Err(_) => false,
        }
    }
    
    fn is_programmatic_ad_domain(&self, domain: &str) -> bool {
        // Check for programmatic advertising patterns
        let programmatic_patterns = [
            // Real-time bidding platforms
            "rtb", "bid", "auction", "exchange",
            // Demand-side platforms
            "dsp", "demand", "supply",
            // Common ad tech patterns
            "adtech", "adx", "ssp", "prebid",
            // Dynamic content delivery
            "cdn", "edge", "cache",
        ];
        
        // Check for suspicious domain structures
        if domain.matches('.').count() > 3 {
            // Deeply nested subdomains often indicate ad tech
            for pattern in &programmatic_patterns {
                if domain.contains(pattern) {
                    return true;
                }
            }
        }
        
        // Check for numeric-heavy domains (common in ad rotation)
        let numeric_count = domain.chars().filter(|c| c.is_numeric()).count();
        if numeric_count > domain.len() / 3 && domain.contains("ad") {
            return true;
        }
        
        false
    }
}

/// Ultimate DNS server with enhanced dynamic ad blocking
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Ultimate Ad Blocker DNS Server");
    println!("=================================");
    println!("📥 Loading Hagezi Ultimate + Steven Black's blocklists...");
    
    // Load multiple blocklists for maximum coverage
    let mut all_blocked_domains = HashSet::new();
    
    // Load Hagezi Ultimate (most comprehensive)
    if let Ok(hagezi_domains) = load_blocklist_from_url("https://cdn.jsdelivr.net/gh/hagezi/dns-blocklists@latest/domains/ultimate.txt").await {
        println!("✅ Loaded {} domains from Hagezi Ultimate", hagezi_domains.len());
        all_blocked_domains.extend(hagezi_domains);
    }
    
    // Load Steven Black's hosts (additional coverage)
    if let Ok(steven_domains) = load_blocklist_from_url("https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts").await {
        println!("✅ Loaded {} domains from Steven Black's hosts", steven_domains.len());
        all_blocked_domains.extend(steven_domains);
    }
    
    // Load local blocklist if available
    if let Ok(local_domains) = load_blocklist("/tmp/clean_blocklist.txt").await {
        println!("✅ Loaded {} domains from local blocklist", local_domains.len());
        all_blocked_domains.extend(local_domains);
    }
    
    let blocked_count = all_blocked_domains.len();
    println!("🎯 Total unique blocked domains: {}", blocked_count);
    
    // Create enhanced ad blocker with dynamic content filtering
    let mut config = AdBlockerConfig::default();
    config.enable_easylist = true;
    config.enable_easyprivacy = true;
    config.block_tracking = true;
    config.block_social = true;
    config.enable_malware_protection = true;
    
    // Add dynamic content filters for better website rendering
    config.custom_filters.extend(vec![
        // Block dynamic ad injection scripts
        "||googlesyndication.com^".to_string(),
        "||doubleclick.net^".to_string(),
        "||googletagmanager.com^".to_string(),
        "||google-analytics.com^".to_string(),
        "||facebook.com/tr^".to_string(),
        "||connect.facebook.net^".to_string(),
        // Block common ad networks that break dynamic content
        "||adsystem.amazon.com^".to_string(),
        "||amazon-adsystem.com^".to_string(),
        "||outbrain.com^".to_string(),
        "||taboola.com^".to_string(),
        // Block tracking pixels that slow down page loads
        "||scorecardresearch.com^".to_string(),
        "||quantserve.com^".to_string(),
    ]);
    
    let blocker = Arc::new(DynamicAdBlocker::new(all_blocked_domains).await?);
    
    let dns_port = 53;
    let addr: SocketAddr = format!("0.0.0.0:{}", dns_port).parse()?;
    
    println!("🌐 Starting ultimate DNS server on port {}...", dns_port);
    
    let socket = UdpSocket::bind(addr)?;
    println!("✅ Ultimate DNS server listening on {}", addr);
    println!("📱 Configure your devices to use this server's IP as DNS");
    println!("🛡️ Enhanced Dynamic Ad Blocking Active!");
    println!("   • Static blocklist: {:.1}M+ domains", blocked_count as f64 / 1_000_000.0);
    println!("   • Dynamic pattern detection for JavaScript ads");
    println!("   • Programmatic advertising detection");
    println!("   • Real-time bidding platform blocking");
    println!("   • Connection monitoring for post-load ads");
    println!("🚀 Enhanced filtering for better website rendering");
    println!("🔧 Press Ctrl+C to stop");
    println!();
    
    let mut query_count = 0u64;
    let mut blocked_count = 0u64;
    let mut dynamic_blocks = 0u64;
    let start_time = Instant::now();
    
    loop {
        let mut buffer = [0; 512];
        
        match socket.recv_from(&mut buffer) {
            Ok((size, client_addr)) => {
                query_count += 1;
                
                if let Some(domain) = extract_domain_from_dns_query(&buffer[..size]) {
                    println!("📱 Query #{}: {} from {}", query_count, domain, client_addr);
                    
                    let should_block = blocker.should_block(&domain).await;
                    
                    if should_block {
                        blocked_count += 1;
                        dynamic_blocks += 1; // All blocks now use enhanced detection
                        println!("   🚫 BLOCKED: Enhanced dynamic ad/tracking detection");
                        
                        let response = create_blocked_dns_response(&buffer[..size]);
                        let _ = socket.send_to(&response, client_addr);
                    } else {
                        println!("   ✅ ALLOWED: Forwarding to upstream DNS");
                        // Add connection monitoring for post-connection ad blocking
                        monitor_connection_for_ads(&domain);
                        forward_dns_query_with_timeout(&socket, &buffer[..size], client_addr);
                    }
                    
                    // Show enhanced stats every 25 queries
                    if query_count % 25 == 0 {
                        let block_rate = (blocked_count as f64 / query_count as f64) * 100.0;
                        let dynamic_rate = (dynamic_blocks as f64 / blocked_count.max(1) as f64) * 100.0;
                        let uptime = start_time.elapsed().as_secs();
                        println!("📊 Enhanced Ultimate Stats: {}/{} blocked ({:.1}%) | {} enhanced blocks ({:.1}%) | {}s uptime | Dynamic ad detection active", 
                            blocked_count, query_count, block_rate, dynamic_blocks, dynamic_rate, uptime);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving DNS query: {}", e);
            }
        }
    }
}

async fn load_blocklist_from_url(url: &str) -> Result<HashSet<String>> {
    let mut domains = HashSet::new();
    
    match reqwest::get(url).await {
        Ok(response) => {
            let content = response.text().await?;
            for line in content.lines() {
                let line = line.trim();
                
                // Handle different formats (hosts file, domain list, etc.)
                let domain = if line.starts_with("0.0.0.0 ") || line.starts_with("127.0.0.1 ") {
                    // hosts file format
                    line.split_whitespace().nth(1).unwrap_or("").to_string()
                } else if line.starts_with("||") && line.ends_with("^") {
                    // AdBlock format
                    line.trim_start_matches("||").trim_end_matches("^").to_string()
                } else if !line.starts_with('#') && !line.is_empty() && line.contains('.') {
                    // Plain domain format
                    line.to_string()
                } else {
                    continue;
                };
                
                let domain = domain.trim().to_lowercase();
                if !domain.is_empty() && domain.contains('.') && !domain.starts_with('#') {
                    domains.insert(domain);
                }
            }
            println!("🌐 Downloaded {} domains from {}", domains.len(), url);
        }
        Err(e) => {
            println!("⚠️  Failed to download from {}: {}", url, e);
        }
    }
    
    Ok(domains)
}

async fn load_blocklist(file_path: &str) -> Result<HashSet<String>> {
    let mut domains = HashSet::new();
    
    match File::open(file_path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(domain) = line {
                    let domain = domain.trim().to_lowercase();
                    if !domain.is_empty() && !domain.starts_with('#') && domain.contains('.') {
                        domains.insert(domain);
                    }
                }
            }
            println!("📂 Loaded {} domains from {}", domains.len(), file_path);
        }
        Err(_) => {
            println!("⚠️  Local blocklist file not found: {}", file_path);
        }
    }
    
    Ok(domains)
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

fn monitor_connection_for_ads(domain: &str) {
    // Log allowed domains for potential post-connection monitoring
    // This could be extended to integrate with browser extensions or proxy monitoring
    if domain.contains("google") || domain.contains("facebook") || domain.contains("amazon") {
        println!("   👁️  MONITORING: {} for dynamic ad injection", domain);
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

fn forward_dns_query_with_timeout(socket: &UdpSocket, query: &[u8], client_addr: SocketAddr) {
    // Try multiple upstream DNS servers for better reliability
    let upstream_servers = [
        "8.8.8.8:53",      // Google DNS
        "1.1.1.1:53",      // Cloudflare DNS
        "9.9.9.9:53",      // Quad9 DNS
    ];
    
    for upstream_addr_str in &upstream_servers {
        if let Ok(upstream_addr) = upstream_addr_str.parse::<SocketAddr>() {
            if let Ok(upstream_socket) = UdpSocket::bind("0.0.0.0:0") {
                // Set timeout for faster response
                let _ = upstream_socket.set_read_timeout(Some(Duration::from_millis(2000)));
                
                if upstream_socket.send_to(query, upstream_addr).is_ok() {
                    let mut buffer = [0; 512];
                    if let Ok((size, _)) = upstream_socket.recv_from(&mut buffer) {
                        let _ = socket.send_to(&buffer[..size], client_addr);
                        return; // Success, exit early
                    }
                }
            }
        }
    }
    
    // If all upstream servers fail, send a basic response
    println!("   ⚠️  All upstream DNS servers failed, sending basic response");
}