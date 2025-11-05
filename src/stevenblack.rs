use anyhow::Result;
use reqwest;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// StevenBlack hosts file integration
pub struct StevenBlackBlocker {
    blocked_domains: Arc<RwLock<HashSet<String>>>,
    stats: Arc<RwLock<BlockStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct BlockStats {
    pub total_checks: u64,
    pub blocked_domains: u64,
    pub allowed_domains: u64,
    pub hosts_loaded: u64,
}

impl StevenBlackBlocker {
    /// Create new StevenBlack blocker
    pub async fn new() -> Result<Self> {
        let blocker = Self {
            blocked_domains: Arc::new(RwLock::new(HashSet::new())),
            stats: Arc::new(RwLock::new(BlockStats::default())),
        };
        
        // Load default hosts file
        blocker.load_stevenblack_hosts().await?;
        
        Ok(blocker)
    }
    
    /// Load StevenBlack hosts file
    pub async fn load_stevenblack_hosts(&self) -> Result<()> {
        println!("📥 Loading StevenBlack hosts file...");
        
        let url = "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts";
        let response = reqwest::get(url).await?;
        let content = response.text().await?;
        
        let mut blocked_domains = self.blocked_domains.write().await;
        let mut count = 0;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse hosts file format: "0.0.0.0 domain.com"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let ip = parts[0];
                let domain = parts[1];
                
                // Only block domains that point to 0.0.0.0 or 127.0.0.1
                if ip == "0.0.0.0" || ip == "127.0.0.1" {
                    blocked_domains.insert(domain.to_lowercase());
                    count += 1;
                }
            }
        }
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.hosts_loaded = count;
        }
        
        println!("✅ Loaded {} blocked domains from StevenBlack hosts", count);
        Ok(())
    }
    
    /// Check if domain should be blocked
    pub async fn is_blocked(&self, domain: &str) -> bool {
        let mut stats = self.stats.write().await;
        stats.total_checks += 1;
        
        let domain_lower = domain.to_lowercase();
        let blocked_domains = self.blocked_domains.read().await;
        
        // Check exact match
        if blocked_domains.contains(&domain_lower) {
            stats.blocked_domains += 1;
            return true;
        }
        
        // Check subdomains (e.g., if "ads.example.com" is blocked, block "banner.ads.example.com")
        let parts: Vec<&str> = domain_lower.split('.').collect();
        for i in 1..parts.len() {
            let parent_domain = parts[i..].join(".");
            if blocked_domains.contains(&parent_domain) {
                stats.blocked_domains += 1;
                return true;
            }
        }
        
        stats.allowed_domains += 1;
        false
    }
    
    /// Check if URL should be blocked
    pub async fn is_url_blocked(&self, url: &str) -> bool {
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(domain) = parsed_url.domain() {
                return self.is_blocked(domain).await;
            }
        }
        false
    }
    
    /// Get statistics
    pub async fn get_stats(&self) -> BlockStats {
        self.stats.read().await.clone()
    }
    
    /// Add custom blocked domain
    pub async fn add_blocked_domain(&self, domain: &str) {
        let mut blocked_domains = self.blocked_domains.write().await;
        blocked_domains.insert(domain.to_lowercase());
    }
    
    /// Remove domain from blocklist
    pub async fn remove_blocked_domain(&self, domain: &str) {
        let mut blocked_domains = self.blocked_domains.write().await;
        blocked_domains.remove(&domain.to_lowercase());
    }
    
    /// Load additional hosts files
    pub async fn load_additional_hosts(&self, urls: Vec<&str>) -> Result<()> {
        for url in urls {
            println!("📥 Loading additional hosts from: {}", url);
            
            match reqwest::get(url).await {
                Ok(response) => {
                    if let Ok(content) = response.text().await {
                        let mut blocked_domains = self.blocked_domains.write().await;
                        let mut count = 0;
                        
                        for line in content.lines() {
                            let line = line.trim();
                            if line.is_empty() || line.starts_with('#') {
                                continue;
                            }
                            
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                let ip = parts[0];
                                let domain = parts[1];
                                
                                if ip == "0.0.0.0" || ip == "127.0.0.1" {
                                    blocked_domains.insert(domain.to_lowercase());
                                    count += 1;
                                }
                            }
                        }
                        
                        println!("✅ Loaded {} additional domains from {}", count, url);
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to load hosts from {}: {}", url, e);
                }
            }
        }
        
        Ok(())
    }
}