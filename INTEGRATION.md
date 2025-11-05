# 🔧 Integration Guide

## Quick Integration Examples

### 1. Web Browser Extension

```rust
use ad_blocker_api::prelude::*;

// Initialize once
static BLOCKER: OnceCell<SimpleAdBlocker> = OnceCell::new();

async fn init_blocker() -> Result<()> {
    let blocker = SimpleAdBlocker::new().await?;
    BLOCKER.set(blocker).map_err(|_| anyhow::anyhow!("Already initialized"))?;
    Ok(())
}

// Use in request interceptor
async fn should_block_request(url: &str) -> bool {
    if let Some(blocker) = BLOCKER.get() {
        blocker.is_blocked(url).await
    } else {
        false
    }
}
```

### 2. HTTP Proxy Server

```rust
use ad_blocker_api::prelude::*;
use hyper::{Body, Request, Response, StatusCode};

async fn proxy_handler(req: Request<Body>, blocker: &SimpleAdBlocker) -> Result<Response<Body>> {
    let url = req.uri().to_string();
    
    if blocker.is_blocked(&url).await {
        // Return blocked response
        return Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("X-Blocked-By", "AdBlocker")
            .body(Body::empty())?);
    }
    
    // Forward request normally
    forward_request(req).await
}
```

### 3. Mobile App HTTP Client

```rust
use ad_blocker_api::prelude::*;
use reqwest::Client;

pub struct BlockingHttpClient {
    client: Client,
    blocker: SimpleAdBlocker,
}

impl BlockingHttpClient {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            blocker: SimpleAdBlocker::new().await?,
        })
    }
    
    pub async fn get(&self, url: &str) -> Result<reqwest::Response> {
        if self.blocker.is_blocked(url).await {
            return Err(anyhow::anyhow!("URL blocked by ad blocker"));
        }
        
        Ok(self.client.get(url).send().await?)
    }
}
```

### 4. DNS Server

```rust
use ad_blocker_api::prelude::*;
use hickory_server::server::Request;

async fn dns_handler(request: &Request, blocker: &SimpleAdBlocker) -> bool {
    let domain = request.query().name().to_string();
    let url = format!("https://{}", domain);
    
    blocker.is_blocked(&url).await
}
```

### 5. Custom Configuration

```rust
use ad_blocker_api::prelude::*;

async fn create_custom_blocker() -> Result<SimpleAdBlocker> {
    let config = AdBlockerConfig {
        enable_easylist: true,
        enable_easyprivacy: true,
        block_tracking: true,
        block_social: true,
        custom_filters: vec![
            // Block specific domains
            "||ads.example.com^".to_string(),
            "||tracker.badsite.com^".to_string(),
            
            // Block specific paths
            "*/ads/*".to_string(),
            "*/tracking.js".to_string(),
        ],
        whitelist_domains: vec![
            "trusted-ads.com".to_string(),
            "analytics.mysite.com".to_string(),
        ],
        ..Default::default()
    };
    
    SimpleAdBlocker::with_config(config).await
}
```

## Performance Tips

### 1. Reuse Blocker Instances
```rust
// ✅ Good - reuse instance
let blocker = SimpleAdBlocker::new().await?;
for url in urls {
    blocker.is_blocked(&url).await;
}

// ❌ Bad - creates new instance each time
for url in urls {
    let blocker = SimpleAdBlocker::new().await?;
    blocker.is_blocked(&url).await;
}
```

### 2. Batch Processing
```rust
// Check multiple URLs at once
let results = blocker.batch_check(urls, Some("https://example.com")).await?;
```

### 3. Use Minimal Config for Performance
```rust
let config = AdBlockerConfig::performance_focused();
let blocker = SimpleAdBlocker::with_config(config).await?;
```

## Error Handling

```rust
match blocker.check_url(url).await {
    Ok(result) => {
        if result.should_block {
            println!("Blocked: {} ({})", result.reason, result.category);
        }
    }
    Err(e) => {
        eprintln!("Error checking URL: {}", e);
        // Fail open - don't block on errors
    }
}
```

## Thread Safety

The blocker is not `Send + Sync` due to the underlying adblock engine. For multi-threaded applications:

```rust
// Option 1: Create per-thread instances
tokio::task_local! {
    static BLOCKER: SimpleAdBlocker;
}

// Option 2: Use a single-threaded runtime
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let blocker = SimpleAdBlocker::new().await?;
    // Use blocker...
}
```

## Testing

```rust
#[tokio::test]
async fn test_ad_blocking() {
    let blocker = SimpleAdBlocker::new().await.unwrap();
    
    // Test known ad URL
    assert!(blocker.is_blocked("https://googleads.g.doubleclick.net/ads").await);
    
    // Test clean URL
    assert!(!blocker.is_blocked("https://github.com").await);
}
```