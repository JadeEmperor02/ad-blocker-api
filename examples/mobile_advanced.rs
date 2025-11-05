use ad_blocker_api::prelude::*;
use anyhow::Result;
use reqwest::{Client, Response, Method};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Advanced mobile HTTP client with comprehensive ad blocking
pub struct AdvancedMobileClient {
    client: Client,
    blocker: SimpleAdBlocker,
    stats: AdvancedStats,
    settings: MobileSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedStats {
    pub session_start: std::time::SystemTime,
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub allowed_requests: u64,
    pub bytes_saved: u64,
    pub time_saved_ms: u64,
    pub blocked_by_category: HashMap<String, u64>,
    pub top_blocked_domains: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileSettings {
    pub aggressive_blocking: bool,
    pub block_social_media: bool,
    pub block_analytics: bool,
    pub save_bandwidth: bool,
    pub offline_mode: bool,
}

impl Default for AdvancedStats {
    fn default() -> Self {
        Self {
            session_start: std::time::SystemTime::now(),
            total_requests: 0,
            blocked_requests: 0,
            allowed_requests: 0,
            bytes_saved: 0,
            time_saved_ms: 0,
            blocked_by_category: HashMap::new(),
            top_blocked_domains: HashMap::new(),
        }
    }
}

impl Default for MobileSettings {
    fn default() -> Self {
        Self {
            aggressive_blocking: false,
            block_social_media: true,
            block_analytics: true,
            save_bandwidth: true,
            offline_mode: false,
        }
    }
}

impl AdvancedMobileClient {
    /// Create new advanced mobile client
    pub async fn new() -> Result<Self> {
        Self::with_settings(MobileSettings::default()).await
    }
    
    /// Create with custom settings
    pub async fn with_settings(settings: MobileSettings) -> Result<Self> {
        let config = AdBlockerConfig {
            enable_easylist: true,
            enable_easyprivacy: settings.block_analytics,
            block_tracking: settings.block_analytics,
            block_social: settings.block_social_media,
            aggressive_blocking: settings.aggressive_blocking,
            custom_filters: vec![
                // Mobile-specific patterns
                "||ads.*.com^".to_string(),
                "||*.ads.*.com^".to_string(),
                "*/mobile-ads/*".to_string(),
                "*/app-tracking/*".to_string(),
                "||crashlytics.com^".to_string(),
                "||flurry.com^".to_string(),
                
                // Social media tracking
                "||connect.facebook.net^".to_string(),
                "||platform.twitter.com^".to_string(),
                "||platform.linkedin.com^".to_string(),
                
                // Mobile analytics
                "||google-analytics.com^".to_string(),
                "||googletagmanager.com^".to_string(),
                "||mixpanel.com^".to_string(),
                "||amplitude.com^".to_string(),
            ],
            whitelist_domains: vec![
                // Essential mobile services
                "api.github.com".to_string(),
                "api.twitter.com".to_string(),
                "graph.facebook.com".to_string(),
                "api.instagram.com".to_string(),
                "api.linkedin.com".to_string(),
                "maps.googleapis.com".to_string(),
                "fcm.googleapis.com".to_string(), // Firebase messaging
            ],
            ..Default::default()
        };
        
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("MobileApp/2.0 (iOS; AdBlocker)");
            
        // Bandwidth saving features are enabled by default in reqwest
        // (gzip, deflate, brotli compression)
        
        let client = client_builder.build()?;
        let blocker = SimpleAdBlocker::with_config(config).await?;
        
        Ok(Self {
            client,
            blocker,
            stats: AdvancedStats::default(),
            settings,
        })
    }
    
    /// Make HTTP request with comprehensive blocking
    pub async fn request(&mut self, method: Method, url: &str) -> Result<MobileResponse> {
        let start_time = Instant::now();
        self.stats.total_requests += 1;
        
        // Parse URL for domain tracking
        let parsed_url = url::Url::parse(url)?;
        let domain = parsed_url.domain().unwrap_or("unknown").to_string();
        
        // Check blocking
        let block_result = self.blocker.check_url(url).await?;
        
        if block_result.should_block {
            self.stats.blocked_requests += 1;
            self.stats.time_saved_ms += 150; // Estimated time saved
            self.stats.bytes_saved += self.estimate_blocked_bytes(&block_result.category);
            
            // Track blocking categories
            let category = format!("{:?}", block_result.category);
            *self.stats.blocked_by_category.entry(category).or_insert(0) += 1;
            *self.stats.top_blocked_domains.entry(domain).or_insert(0) += 1;
            
            let bytes_saved = self.estimate_blocked_bytes(&block_result.category);
            return Ok(MobileResponse::Blocked {
                url: url.to_string(),
                reason: block_result.reason,
                category: block_result.category,
                time_saved_ms: 150,
                bytes_saved,
            });
        }
        
        // Make actual request
        self.stats.allowed_requests += 1;
        
        let request = match method {
            Method::GET => self.client.get(url),
            Method::POST => self.client.post(url),
            Method::PUT => self.client.put(url),
            Method::DELETE => self.client.delete(url),
            _ => return Err(anyhow::anyhow!("Unsupported method: {}", method)),
        };
        
        let response = request.send().await?;
        let elapsed = start_time.elapsed();
        
        Ok(MobileResponse::Success {
            response,
            elapsed_ms: elapsed.as_millis() as u64,
            domain,
        })
    }
    
    /// Convenience methods
    pub async fn get(&mut self, url: &str) -> Result<MobileResponse> {
        self.request(Method::GET, url).await
    }
    
    pub async fn post(&mut self, url: &str) -> Result<MobileResponse> {
        self.request(Method::POST, url).await
    }
    
    /// Get comprehensive statistics
    pub fn get_stats(&self) -> &AdvancedStats {
        &self.stats
    }
    
    /// Get settings
    pub fn get_settings(&self) -> &MobileSettings {
        &self.settings
    }
    
    /// Update settings (requires restart)
    pub fn update_settings(&mut self, settings: MobileSettings) {
        self.settings = settings;
    }
    
    /// Generate stats report
    pub fn generate_report(&self) -> serde_json::Value {
        let session_duration = self.stats.session_start.elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
            
        json!({
            "session": {
                "duration_seconds": session_duration,
                "total_requests": self.stats.total_requests,
                "blocked_requests": self.stats.blocked_requests,
                "allowed_requests": self.stats.allowed_requests,
                "block_rate_percent": if self.stats.total_requests > 0 {
                    (self.stats.blocked_requests as f64 / self.stats.total_requests as f64) * 100.0
                } else { 0.0 }
            },
            "savings": {
                "bytes_saved": self.stats.bytes_saved,
                "bytes_saved_mb": self.stats.bytes_saved as f64 / (1024.0 * 1024.0),
                "time_saved_ms": self.stats.time_saved_ms,
                "estimated_cost_saved_usd": (self.stats.bytes_saved as f64 / (1024.0 * 1024.0)) * 0.01 // $0.01 per MB
            },
            "categories": self.stats.blocked_by_category,
            "top_blocked_domains": self.stats.top_blocked_domains,
            "settings": self.settings
        })
    }
    
    fn estimate_blocked_bytes(&self, category: &BlockCategory) -> u64 {
        match category {
            BlockCategory::Advertisement => 25000,  // 25KB average ad
            BlockCategory::Tracking => 5000,       // 5KB tracking script
            BlockCategory::Social => 15000,        // 15KB social widget
            BlockCategory::Malware => 10000,       // 10KB malicious content
            _ => 8000,                              // 8KB default
        }
    }
}

/// Response from mobile client
#[derive(Debug)]
pub enum MobileResponse {
    Success {
        response: Response,
        elapsed_ms: u64,
        domain: String,
    },
    Blocked {
        url: String,
        reason: String,
        category: BlockCategory,
        time_saved_ms: u64,
        bytes_saved: u64,
    },
}

impl MobileResponse {
    pub fn is_blocked(&self) -> bool {
        matches!(self, MobileResponse::Blocked { .. })
    }
    
    pub fn is_success(&self) -> bool {
        matches!(self, MobileResponse::Success { .. })
    }
}

/// Demo mobile app with advanced features
#[tokio::main]
async fn main() -> Result<()> {
    println!("📱 Advanced Mobile App Demo");
    println!("============================\n");
    
    // Create advanced client with custom settings
    let settings = MobileSettings {
        aggressive_blocking: true,
        block_social_media: true,
        block_analytics: true,
        save_bandwidth: true,
        offline_mode: false,
    };
    
    let mut client = AdvancedMobileClient::with_settings(settings).await?;
    
    println!("⚙️  Client Settings:");
    println!("   Aggressive blocking: {}", client.get_settings().aggressive_blocking);
    println!("   Block social media: {}", client.get_settings().block_social_media);
    println!("   Block analytics: {}", client.get_settings().block_analytics);
    println!("   Save bandwidth: {}", client.get_settings().save_bandwidth);
    println!();
    
    // Simulate mobile app workflow
    let mobile_requests = vec![
        // App startup requests
        ("https://api.github.com/user", "User Profile API"),
        ("https://api.openweathermap.org/data/2.5/weather", "Weather API"),
        ("https://jsonplaceholder.typicode.com/posts", "Content API"),
        
        // Tracking/Analytics (will be blocked)
        ("https://google-analytics.com/collect", "Google Analytics"),
        ("https://mixpanel.com/track", "Mixpanel Analytics"),
        ("https://amplitude.com/api/2/httpapi", "Amplitude Analytics"),
        ("https://crashlytics.com/api/crash", "Crash Reporting"),
        
        // Ads (will be blocked)
        ("https://ads.google.com/mobile/banner", "Mobile Banner Ad"),
        ("https://facebook.com/tr?id=mobile", "Facebook Pixel"),
        ("https://doubleclick.net/mobile/interstitial", "Interstitial Ad"),
        
        // Social widgets (will be blocked)
        ("https://connect.facebook.net/en_US/sdk.js", "Facebook SDK"),
        ("https://platform.twitter.com/widgets.js", "Twitter Widget"),
        ("https://platform.linkedin.com/in.js", "LinkedIn Widget"),
        
        // Legitimate APIs
        ("https://maps.googleapis.com/maps/api/geocode/json", "Maps API"),
        ("https://fcm.googleapis.com/fcm/send", "Push Notifications"),
        ("https://graph.facebook.com/me", "Facebook Graph API"),
    ];
    
    println!("🔄 Simulating mobile app requests...\n");
    
    for (url, description) in mobile_requests {
        println!("📡 {}: {}", description, url);
        
        match client.get(url).await? {
            MobileResponse::Success { response, elapsed_ms, domain } => {
                println!("   ✅ Success: {} {} ({}ms, {})", 
                    response.status(), 
                    response.status().canonical_reason().unwrap_or(""),
                    elapsed_ms,
                    domain
                );
            }
            MobileResponse::Blocked { reason, category, time_saved_ms, bytes_saved, .. } => {
                println!("   🚫 Blocked: {} ({:?})", reason, category);
                println!("      Saved: {}ms, {} bytes", time_saved_ms, bytes_saved);
            }
        }
        
        println!();
        sleep(Duration::from_millis(300)).await;
    }
    
    // Generate comprehensive report
    let report = client.generate_report();
    println!("📊 Session Report:");
    println!("{}", serde_json::to_string_pretty(&report)?);
    
    println!("\n🎯 Key Benefits:");
    let stats = client.get_stats();
    println!("   • Blocked {} malicious/unwanted requests", stats.blocked_requests);
    println!("   • Saved {:.2} MB of bandwidth", stats.bytes_saved as f64 / (1024.0 * 1024.0));
    println!("   • Saved {}ms of loading time", stats.time_saved_ms);
    println!("   • Estimated cost savings: ${:.3}", (stats.bytes_saved as f64 / (1024.0 * 1024.0)) * 0.01);
    
    println!("\n✨ Advanced mobile ad blocking complete!");
    
    Ok(())
}