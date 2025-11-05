use ad_blocker_api::prelude::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🛡️  Ad Blocker API Demo");
    
    // Create ad blocker with default config
    let blocker = SimpleAdBlocker::new().await?;
    
    // Test URLs
    let test_urls = vec![
        "https://www.google.com",
        "https://googleads.g.doubleclick.net/pagead/ads",
        "https://www.google-analytics.com/analytics.js",
        "https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js",
        "https://github.com",
        "https://facebook.com/tr?id=123456789",
    ];
    
    println!("\n📋 Testing URLs:");
    for url in test_urls {
        let result = blocker.check_url(url).await?;
        let status = if result.should_block { "🚫 BLOCKED" } else { "✅ ALLOWED" };
        println!("{} - {} ({})", status, url, result.reason);
    }
    
    // Example of advanced usage
    println!("\n🔧 Advanced Usage Example:");
    let mut config = AdBlockerConfig::default();
    config.custom_filters.push("||example-ads.com^".to_string());
    config.whitelist_domains.push("trusted-site.com".to_string());
    
    let advanced_blocker = SimpleAdBlocker::with_config(config).await?;
    
    let custom_test = advanced_blocker.check_url("https://example-ads.com/banner.jpg").await?;
    println!("Custom filter test: {} - {}", 
        if custom_test.should_block { "🚫 BLOCKED" } else { "✅ ALLOWED" },
        custom_test.reason
    );
    
    println!("\n✨ Ad Blocker API is ready for integration!");
    
    Ok(())
}
