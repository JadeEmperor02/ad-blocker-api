use ad_blocker_api::prelude::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🛡️  Basic Ad Blocker Usage Example\n");
    
    // Create a simple ad blocker
    let blocker = SimpleAdBlocker::new().await?;
    
    // Test some URLs
    let test_urls = vec![
        "https://www.example.com",
        "https://googleads.g.doubleclick.net/pagead/ads",
        "https://www.google-analytics.com/analytics.js",
        "https://github.com",
        "https://facebook.com/tr?id=123456789",
    ];
    
    for url in test_urls {
        let is_blocked = blocker.is_blocked(url).await;
        let status = if is_blocked { "🚫 BLOCKED" } else { "✅ ALLOWED" };
        println!("{} - {}", status, url);
    }
    
    // Get detailed information
    println!("\n📊 Detailed Check:");
    let result = blocker.check_url("https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js").await?;
    println!("URL: https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js");
    println!("Blocked: {}", result.should_block);
    println!("Reason: {}", result.reason);
    println!("Category: {:?}", result.category);
    
    // Show statistics
    let stats = blocker.get_stats().await;
    println!("\n📈 Statistics:");
    println!("Total requests: {}", stats.total_requests);
    println!("Blocked requests: {}", stats.blocked_requests);
    println!("Block percentage: {:.1}%", stats.block_percentage());
    
    Ok(())
}