# 🛡️ Ad Blocker API

A comprehensive, high-performance ad-blocking library for Rust applications. Block advertisements, tracking scripts, malware, and social media widgets with ease.

## ✨ Features

- **Multiple Filter Sources**: EasyList, EasyPrivacy, malware protection, and social media blocking
- **Custom Filters**: Add your own blocking rules
- **Whitelist Support**: Allow specific domains
- **Pattern Matching**: Advanced regex-based blocking for tracking and social media
- **Statistics**: Track blocked requests and performance metrics
- **Async/Await**: Built with Tokio for high performance
- **Easy Integration**: Simple API for quick integration into any application
- **Web Service**: Optional HTTP API for microservice architectures

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ad-blocker-api = { path = "." }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

### Basic Usage

```rust
use ad_blocker_api::prelude::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a simple ad blocker
    let blocker = SimpleAdBlocker::new().await?;
    
    // Check if a URL should be blocked
    if blocker.is_blocked("https://googleads.g.doubleclick.net/ads").await {
        println!("🚫 Ad blocked!");
    }
    
    // Get detailed information
    let result = blocker.check_url("https://www.google-analytics.com/analytics.js").await?;
    println!("Blocked: {}, Reason: {}", result.should_block, result.reason);
    
    Ok(())
}
```

### Advanced Configuration

```rust
use ad_blocker_api::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create custom configuration
    let config = AdBlockerConfig {
        enable_easylist: true,
        enable_easyprivacy: true,
        enable_malware_protection: true,
        block_tracking: true,
        block_social: true,
        custom_filters: vec![
            "||example-ads.com^".to_string(),
            "||tracking-service.net^".to_string(),
        ],
        whitelist_domains: vec![
            "trusted-site.com".to_string(),
        ],
        ..Default::default()
    };
    
    let blocker = SimpleAdBlocker::with_config(config).await?;
    
    // Your blocking logic here...
    
    Ok(())
}
```

## 🔧 Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `enable_easylist` | Enable EasyList ad filters | `true` |
| `enable_easyprivacy` | Enable EasyPrivacy tracking protection | `true` |
| `enable_malware_protection` | Enable malware domain blocking | `true` |
| `block_tracking` | Block tracking scripts | `true` |
| `block_social` | Block social media widgets | `false` |
| `aggressive_blocking` | More aggressive blocking rules | `false` |
| `custom_filters` | Your custom filter rules | `[]` |
| `whitelist_domains` | Domains to never block | `[]` |
| `cache_filters` | Cache downloaded filter lists | `true` |

### Preset Configurations

```rust
// Minimal blocking (fastest)
let config = AdBlockerConfig::minimal();

// Privacy-focused (blocks everything)
let config = AdBlockerConfig::privacy_focused();

// Performance-focused (basic ads only)
let config = AdBlockerConfig::performance_focused();
```

## 📊 Statistics

Track your blocking performance:

```rust
let stats = blocker.get_stats().await;
println!("Blocked {}/{} requests ({:.1}%)", 
    stats.blocked_requests, 
    stats.total_requests,
    stats.block_percentage()
);
```

## 🌐 Web Service

Run as a standalone web service:

```bash
cargo run --example web_server
```

Then use the HTTP API:

```bash
# Check a single URL
curl "http://localhost:8080/check?url=https://example.com"

# Get statistics
curl "http://localhost:8080/stats"

# Web interface
open http://localhost:8080
```

## 🎯 Blocking Categories

The API categorizes blocked content:

- **Advertisement**: Traditional ads and banners
- **Tracking**: Analytics and tracking scripts
- **Malware**: Known malicious domains
- **Social**: Social media widgets and buttons
- **Custom**: Your custom filter rules
- **Whitelisted**: Explicitly allowed content
- **Clean**: Safe, unblocked content

## 🔍 Filter Syntax

Add custom filters using standard adblock syntax:

```rust
// Block entire domain
"||ads.example.com^"

// Block specific path
"||example.com/ads/*"

// Block with wildcards
"*://*/ads.js"

// Element hiding (cosmetic)
"example.com##.advertisement"
```

## 🚀 Performance

- **Async/Await**: Non-blocking operations
- **Filter Caching**: Reuse downloaded filter lists
- **Batch Processing**: Check multiple URLs efficiently
- **Memory Efficient**: Optimized filter storage

## 🛠️ Integration Examples

### Web Browser Extension

```rust
// In your browser extension's content script
let blocker = SimpleAdBlocker::new().await?;

// Check each request
if blocker.is_blocked(&request_url).await {
    // Block the request
    return Err("Blocked by ad blocker");
}
```

### Proxy Server

```rust
// In your HTTP proxy
let result = blocker.should_block(&url, Some(&referrer)).await?;
if result.should_block {
    return HttpResponse::new(204); // No Content
}
```

### Mobile App

```rust
// In your mobile app's HTTP client
if blocker.is_blocked(&api_url).await {
    // Skip the request or show placeholder
    return Ok(PlaceholderResponse::new());
}
```

## 📝 Examples

Run the included examples:

```bash
# Basic usage example
cargo run --example basic_usage

# Web server example
cargo run --example web_server
```

## 🤝 Contributing

Contributions welcome! Please read our contributing guidelines and submit pull requests.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [adblock-rust](https://github.com/brave/adblock-rust) - Core blocking engine
- [EasyList](https://easylist.to/) - Filter lists
- [uBlock Origin](https://github.com/gorhill/uBlock) - Inspiration and filter syntax

---

**Made with ❤️ for a cleaner, faster web**