use ad_blocker_api::prelude::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🛡️  StevenBlack Hosts File Test");
    println!("===============================\n");
    
    // Create StevenBlack blocker
    let blocker = StevenBlackBlocker::new().await?;
    
    // Test domains
    let test_domains = vec![
        // Should be allowed
        ("github.com", "GitHub"),
        ("google.com", "Google"),
        ("stackoverflow.com", "Stack Overflow"),
        ("rust-lang.org", "Rust Language"),
        
        // Should be blocked (common ad/tracking domains)
        ("doubleclick.net", "Google DoubleClick"),
        ("googleadservices.com", "Google Ad Services"),
        ("googlesyndication.com", "Google AdSense"),
        ("facebook.com", "Facebook (tracking)"),
        ("google-analytics.com", "Google Analytics"),
        ("googletagmanager.com", "Google Tag Manager"),
        ("scorecardresearch.com", "Scorecard Research"),
        ("outbrain.com", "Outbrain Ads"),
        ("taboola.com", "Taboola Ads"),
        ("adsystem.com", "Ad System"),
        
        // Malware/suspicious domains (likely blocked)
        ("malware-example.com", "Example Malware Domain"),
        ("phishing-site.net", "Example Phishing Site"),
    ];
    
    println!("🔍 Testing domains against StevenBlack hosts file:\n");
    
    let mut blocked_count = 0;
    let mut allowed_count = 0;
    
    for (domain, description) in test_domains {
        let is_blocked = blocker.is_blocked(domain).await;
        
        if is_blocked {
            blocked_count += 1;
            println!("🚫 BLOCKED: {} ({})", domain, description);
        } else {
            allowed_count += 1;
            println!("✅ ALLOWED: {} ({})", domain, description);
        }
    }
    
    println!("\n📊 Test Results:");
    println!("   Blocked: {} domains", blocked_count);
    println!("   Allowed: {} domains", allowed_count);
    println!("   Block rate: {:.1}%", (blocked_count as f64 / (blocked_count + allowed_count) as f64) * 100.0);
    
    // Show StevenBlack stats
    let stats = blocker.get_stats().await;
    println!("\n📈 StevenBlack Statistics:");
    println!("   Total hosts loaded: {}", stats.hosts_loaded);
    println!("   Total checks performed: {}", stats.total_checks);
    println!("   Domains blocked: {}", stats.blocked_domains);
    println!("   Domains allowed: {}", stats.allowed_domains);
    
    // Test URL blocking
    println!("\n🌐 Testing URL blocking:");
    let test_urls = vec![
        "https://github.com/rust-lang/rust",
        "https://doubleclick.net/ads/banner.jpg",
        "https://google-analytics.com/collect?id=123",
        "https://www.google.com/search?q=rust",
    ];
    
    for url in test_urls {
        let is_blocked = blocker.is_url_blocked(url).await;
        let status = if is_blocked { "🚫 BLOCKED" } else { "✅ ALLOWED" };
        println!("   {} - {}", status, url);
    }
    
    println!("\n✨ StevenBlack integration test complete!");
    println!("💡 Run 'cargo run --example stevenblack_proxy' to start the HTTP proxy");
    
    Ok(())
}