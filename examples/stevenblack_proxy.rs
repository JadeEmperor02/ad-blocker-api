use ad_blocker_api::prelude::*;
use anyhow::Result;
// use bytes::Bytes;
use http::{Method, StatusCode, Uri};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server};
use hyper_tls::HttpsConnector;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// HTTP Proxy server using StevenBlack hosts file
#[derive(Clone)]
struct ProxyService {
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    blocker: Arc<StevenBlackBlocker>,
    stats: Arc<RwLock<ProxyStats>>,
}

#[derive(Debug, Default, Clone)]
struct ProxyStats {
    total_requests: u64,
    blocked_requests: u64,
    forwarded_requests: u64,
    bytes_saved: u64,
}

impl ProxyService {
    async fn new() -> Result<Self> {
        println!("🔄 Initializing StevenBlack ad blocker...");
        let blocker = Arc::new(StevenBlackBlocker::new().await?);
        
        // Load additional blocklists
        let additional_hosts = vec![
            "https://raw.githubusercontent.com/StevenBlack/hosts/master/alternates/fakenews/hosts",
            "https://raw.githubusercontent.com/StevenBlack/hosts/master/alternates/gambling/hosts",
            "https://raw.githubusercontent.com/StevenBlack/hosts/master/alternates/porn/hosts",
        ];
        
        println!("📥 Loading additional blocklists...");
        if let Err(e) = blocker.load_additional_hosts(additional_hosts).await {
            eprintln!("⚠️  Warning: Could not load some additional hosts: {}", e);
        }
        
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        
        Ok(Self {
            client,
            blocker,
            stats: Arc::new(RwLock::new(ProxyStats::default())),
        })
    }
    
    async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        drop(stats);
        
        let uri = req.uri();
        let host = uri.host().unwrap_or("unknown");
        
        println!("📱 Request: {} {}", req.method(), uri);
        
        // Check if domain should be blocked
        if self.blocker.is_blocked(host).await {
            let mut stats = self.stats.write().await;
            stats.blocked_requests += 1;
            stats.bytes_saved += 50000; // Estimate 50KB saved per blocked request
            drop(stats);
            
            println!("   🚫 BLOCKED: {}", host);
            return Ok(self.create_blocked_response(uri));
        }
        
        println!("   ✅ ALLOWED: Forwarding to {}", host);
        
        // Forward the request
        match self.forward_request(req).await {
            Ok(response) => {
                let mut stats = self.stats.write().await;
                stats.forwarded_requests += 1;
                Ok(response)
            }
            Err(e) => {
                eprintln!("   ❌ Error forwarding request: {}", e);
                Ok(Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .body(Body::from("Proxy Error"))
                    .unwrap())
            }
        }
    }
    
    async fn forward_request(&self, mut req: Request<Body>) -> Result<Response<Body>> {
        // Handle CONNECT method for HTTPS
        if req.method() == Method::CONNECT {
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap());
        }
        
        // Ensure we have a proper URI
        let uri = req.uri();
        if uri.scheme().is_none() {
            let new_uri = format!("http://{}", uri)
                .parse::<Uri>()
                .map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?;
            *req.uri_mut() = new_uri;
        }
        
        // Remove hop-by-hop headers
        let headers = req.headers_mut();
        headers.remove("connection");
        headers.remove("proxy-connection");
        headers.remove("proxy-authenticate");
        headers.remove("proxy-authorization");
        headers.remove("te");
        headers.remove("trailers");
        headers.remove("upgrade");
        
        // Forward the request
        let response = self.client.request(req).await
            .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
        
        Ok(response)
    }
    
    fn create_blocked_response(&self, uri: &Uri) -> Response<Body> {
        let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>🛡️ Blocked by StevenBlack</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            text-align: center; 
            padding: 20px; 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            margin: 0;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        .container {{ 
            background: rgba(255,255,255,0.1); 
            padding: 40px; 
            border-radius: 20px; 
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px rgba(0,0,0,0.3);
            max-width: 500px;
            width: 90%;
        }}
        .emoji {{ font-size: 64px; margin-bottom: 20px; }}
        h1 {{ margin: 0 0 10px 0; font-size: 28px; }}
        .domain {{ 
            background: rgba(255,255,255,0.2); 
            padding: 15px; 
            border-radius: 10px; 
            word-break: break-all;
            font-family: monospace;
            margin: 20px 0;
            font-size: 14px;
        }}
        .info {{ opacity: 0.9; font-size: 16px; line-height: 1.5; }}
        .badge {{ 
            display: inline-block;
            background: rgba(255,255,255,0.2);
            padding: 5px 15px;
            border-radius: 20px;
            font-size: 12px;
            margin-top: 20px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="emoji">🛡️</div>
        <h1>Domain Blocked</h1>
        <p class="info">This domain is blocked by StevenBlack's comprehensive hosts file for your protection.</p>
        <div class="domain">{}</div>
        <p class="info">
            <strong>Why was this blocked?</strong><br>
            This domain is known to serve ads, malware, tracking scripts, or other unwanted content.
        </p>
        <div class="badge">Protected by Rust Ad Blocker + StevenBlack</div>
    </div>
</body>
</html>
"#, uri.host().unwrap_or("unknown"));

        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html; charset=utf-8")
            .header("cache-control", "no-cache")
            .body(Body::from(html))
            .unwrap()
    }
    
    async fn get_stats(&self) -> (ProxyStats, ad_blocker_api::stevenblack::BlockStats) {
        let proxy_stats = self.stats.read().await.clone();
        let blocker_stats = self.blocker.get_stats().await;
        (proxy_stats, blocker_stats)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🛡️  StevenBlack HTTP Proxy Server");
    println!("==================================");
    
    // Create proxy service
    let proxy_service = ProxyService::new().await?;
    
    // Get local IP
    let local_ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    let port = 8889;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    println!("🌐 Starting HTTP proxy server...");
    println!("📍 Proxy server: {}:{}", local_ip, port);
    println!();
    println!("📱 PHONE SETUP:");
    println!("1. Connect phone to same WiFi network");
    println!("2. WiFi Settings → HTTP Proxy → Manual");
    println!("3. Server: {}", local_ip);
    println!("4. Port: {}", port);
    println!("5. Save and browse!");
    println!();
    
    // Show initial stats
    let (_, blocker_stats) = proxy_service.get_stats().await;
    println!("📊 Loaded {} blocked domains", blocker_stats.hosts_loaded);
    println!("✅ Proxy server ready! Press Ctrl+C to stop.\n");
    
    // Create service
    let service = proxy_service.clone();
    let make_svc = make_service_fn(move |_conn| {
        let service = service.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let service = service.clone();
                async move { service.handle_request(req).await }
            }))
        }
    });
    
    // Start server
    let server = Server::bind(&addr).serve(make_svc);
    
    // Spawn stats reporter
    let stats_service = proxy_service.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let (proxy_stats, _blocker_stats) = stats_service.get_stats().await;
            
            if proxy_stats.total_requests > 0 {
                let block_rate = (proxy_stats.blocked_requests as f64 / proxy_stats.total_requests as f64) * 100.0;
                println!("📊 Stats: {}/{} blocked ({:.1}%), {} KB saved", 
                    proxy_stats.blocked_requests, 
                    proxy_stats.total_requests,
                    block_rate,
                    proxy_stats.bytes_saved / 1024
                );
            }
        }
    });
    
    // Run server
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
    
    Ok(())
}

fn get_local_ip() -> Option<String> {
    use std::net::{TcpStream};
    
    if let Ok(stream) = TcpStream::connect("8.8.8.8:80") {
        if let Ok(local_addr) = stream.local_addr() {
            return Some(local_addr.ip().to_string());
        }
    }
    
    None
}