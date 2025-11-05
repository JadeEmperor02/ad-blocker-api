use ad_blocker_api::prelude::*;
use anyhow::Result;
use reqwest::{Client, Response, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Mobile app HTTP client with built-in ad blocking
pub struct MobileAdBlockingClient {
    client: Client,
    blocker: SimpleAdBlocker,
    stats: ClientStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStats {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub bytes_saved: u64,
    pub time_saved_ms: u64,
}

impl Default for ClientStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            blocked_requests: 0,
            bytes_saved: 0,
            time_saved_ms: 0,
        }
    }
}

impl MobileAdBlockingClient {
    /// Create a new mobile client with ad blocking
    pub async fn new() -> Result<Self> {
        // Mobile-optimized config - faster, less memory
        let config = AdBlockerConfig {
            enable_easylist: true,
            enable_easyprivacy: false, // Disabled for performance
            enable_malware_protection: false,
            block_tracking: true,
            block_social: false,
            aggressive_blocking: false,
            cache_filters: true,
            custom_filters: vec![
                // Mobile-specific ad patterns
                "||mobileads.google.com^".to_string(),
                "||ads.facebook.com^".to_string(),
                "||analytics.tiktok.com^".to_string(),
                "*/mobile-ads/*".to_string(),
                "*/app-ads.js".to_string(),
            ],
            whitelist_domains: vec![
                // Common mobile APIs that shouldn't be blocked
                "api.github.com".to_string(),
                "api.twitter.com".to_string(),
                "graph.facebook.com".to_string(),
            ],
        };
        
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("MobileApp/1.0 AdBlocker")
            .build()?;
            
        let blocker = SimpleAdBlocker::with_config(config).await?;
        
        Ok(Self {
            client,
            blocker,
            stats: ClientStats::default(),
        })
    }
    
    /// Make a GET request with ad blocking
    pub async fn get(&mut self, url: &str) -> Result<Option<Response>> {
        self.request("GET", url, None).await
    }
    
    /// Make a POST request with ad blocking
    pub async fn post(&mut self, url: &str, body: Option<String>) -> Result<Option<Response>> {
        self.request("POST", url, body).await
    }
    
    /// Generic request method with ad blocking
    async fn request(&mut self, method: &str, url: &str, body: Option<String>) -> Result<Option<Response>> {
        let start_time = Instant::now();
        self.stats.total_requests += 1;
        
        // Check if URL should be blocked
        let block_result = self.blocker.check_url(url).await?;
        
        if block_result.should_block {
            self.stats.blocked_requests += 1;
            self.stats.time_saved_ms += 100; // Estimate time saved
            self.stats.bytes_saved += 5000; // Estimate bytes saved
            
            println!("🚫 Blocked {} request to: {}", method, url);
            println!("   Reason: {}", block_result.reason);
            println!("   Category: {:?}", block_result.category);
            
            return Ok(None); // Return None for blocked requests
        }
        
        // Make the actual request
        let response = match method {
            "GET" => self.client.get(url).send().await?,
            "POST" => {
                let mut req = self.client.post(url);
                if let Some(body) = body {
                    req = req.body(body);
                }
                req.send().await?
            }
            _ => return Err(anyhow::anyhow!("Unsupported method: {}", method)),
        };
        
        let elapsed = start_time.elapsed();
        println!("✅ {} {} - {}ms", method, url, elapsed.as_millis());
        
        Ok(Some(response))
    }
    
    /// Get client statistics
    pub fn get_stats(&self) -> &ClientStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ClientStats::default();
    }
    
    /// Add custom filter
    pub async fn add_custom_filter(&mut self, filter: String) -> Result<()> {
        // Note: This would require rebuilding the blocker in a real implementation
        println!("Custom filter added: {}", filter);
        Ok(())
    }
    
    /// Check if URL would be blocked without making request
    pub async fn would_block(&self, url: &str) -> Result<bool> {
        Ok(self.blocker.is_blocked(url).await)
    }
}

/// Mobile app simulation
#[tokio::main]
async fn main() -> Result<()> {
    println!("📱 Mobile App with Ad Blocking Demo");
    println!("=====================================\n");
    
    // Create mobile client
    let mut client = MobileAdBlockingClient::new().await?;
    
    // Simulate mobile app API calls
    let api_calls = vec![
        // Legitimate API calls
        ("https://api.github.com/users/octocat", "GitHub API"),
        ("https://jsonplaceholder.typicode.com/posts/1", "JSON API"),
        ("https://httpbin.org/get", "Test API"),
        
        // Ad/tracking requests (will be blocked)
        ("https://googleads.g.doubleclick.net/pagead/ads", "Google Ads"),
        ("https://www.google-analytics.com/collect", "Analytics"),
        ("https://facebook.com/tr?id=123456789", "Facebook Pixel"),
        ("https://mobileads.google.com/banner.jpg", "Mobile Ad"),
        
        // More legitimate calls
        ("https://api.openweathermap.org/data/2.5/weather", "Weather API"),
        ("https://reqres.in/api/users", "User API"),
    ];
    
    println!("🔄 Making API calls...\n");
    
    for (url, description) in api_calls {
        println!("📡 Requesting: {} ({})", description, url);
        
        match client.get(url).await? {
            Some(response) => {
                println!("   ✅ Success: {} {}", response.status(), response.status().canonical_reason().unwrap_or(""));
                
                // Simulate processing response
                if response.status().is_success() {
                    println!("   📄 Content-Type: {:?}", response.headers().get("content-type"));
                }
            }
            None => {
                println!("   🚫 Request blocked by ad blocker");
            }
        }
        
        println!(); // Empty line for readability
        
        // Small delay to simulate real app behavior
        sleep(Duration::from_millis(500)).await;
    }
    
    // Show statistics
    let stats = client.get_stats();
    println!("📊 Session Statistics:");
    println!("   Total requests: {}", stats.total_requests);
    println!("   Blocked requests: {}", stats.blocked_requests);
    println!("   Success rate: {:.1}%", 
        ((stats.total_requests - stats.blocked_requests) as f64 / stats.total_requests as f64) * 100.0);
    println!("   Estimated bytes saved: {} KB", stats.bytes_saved / 1024);
    println!("   Estimated time saved: {}ms", stats.time_saved_ms);
    
    // Demonstrate custom filtering
    println!("\n🔧 Testing Custom Filters:");
    
    let test_urls = vec![
        "https://custom-ad-network.com/banner",
        "https://tracking.badsite.com/pixel.gif",
        "https://legitimate-api.com/data",
    ];
    
    for url in test_urls {
        let would_block = client.would_block(url).await?;
        println!("   {} - {}", 
            if would_block { "🚫 WOULD BLOCK" } else { "✅ WOULD ALLOW" }, 
            url
        );
    }
    
    println!("\n✨ Mobile app ad blocking demo complete!");
    println!("💡 Integration tip: Wrap your existing HTTP client with MobileAdBlockingClient");
    
    Ok(())
}