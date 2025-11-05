use ad_blocker_api::prelude::*;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Simple HTTP proxy for mobile devices
#[tokio::main]
async fn main() -> Result<()> {
    println!("📱 Simple Mobile Ad-Blocking Proxy");
    println!("==================================");
    
    // Create ad blocker
    let blocker = SimpleAdBlocker::new().await?;
    
    // Get local IP address
    let local_ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    let port = 8889;
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    
    println!("🌐 Starting proxy server...");
    println!("📍 Server address: http://{}:{}", local_ip, port);
    println!();
    println!("📱 PHONE SETUP INSTRUCTIONS:");
    println!("1. Connect your phone to the same WiFi network as this computer");
    println!("2. Go to WiFi settings on your phone");
    println!("3. Configure HTTP Proxy (Manual):");
    println!("   • Server: {}", local_ip);
    println!("   • Port: {}", port);
    println!("4. Save and test browsing!");
    println!();
    
    let listener = TcpListener::bind(addr).await?;
    println!("✅ Proxy server running! Press Ctrl+C to stop.");
    println!();
    
    let mut request_count = 0u64;
    let mut blocked_count = 0u64;
    
    loop {
        let (mut stream, _) = listener.accept().await?;
        request_count += 1;
        
        // Read HTTP request
        let mut buffer = [0; 4096];
        match stream.read(&mut buffer).await {
            Ok(n) if n > 0 => {
                let request = String::from_utf8_lossy(&buffer[..n]);
                
                if let Some((method, url)) = parse_request(&request) {
                    println!("📱 Request #{}: {} {}", request_count, method, url);
                    
                    // Check if should be blocked
                    match blocker.check_url(&url).await {
                        Ok(block_result) => {
                            if block_result.should_block {
                                blocked_count += 1;
                                println!("   🚫 BLOCKED: {}", block_result.reason);
                                
                                // Send blocked response
                                let response = create_blocked_response(&url);
                                let _ = stream.write_all(response.as_bytes()).await;
                            } else {
                                println!("   ✅ ALLOWED");
                                
                                // For CONNECT requests (HTTPS), establish tunnel
                                if method == "CONNECT" {
                                    handle_connect(&mut stream, &url).await;
                                } else {
                                    // For HTTP requests, send a simple response
                                    let response = create_allowed_response(&url);
                                    let _ = stream.write_all(response.as_bytes()).await;
                                }
                            }
                        }
                        Err(e) => {
                            println!("   ❌ Error checking URL: {}", e);
                            let response = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
                            let _ = stream.write_all(response.as_bytes()).await;
                        }
                    }
                    
                    // Show stats every 10 requests
                    if request_count % 10 == 0 {
                        let block_rate = (blocked_count as f64 / request_count as f64) * 100.0;
                        println!("📊 Stats: {}/{} requests blocked ({:.1}%)", 
                            blocked_count, request_count, block_rate);
                    }
                }
            }
            _ => {
                // Connection closed or error
                continue;
            }
        }
    }
}

fn parse_request(request: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = request.lines().collect();
    if lines.is_empty() {
        return None;
    }
    
    let first_line = lines[0];
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    
    if parts.len() >= 2 {
        let method = parts[0].to_string();
        let url_part = parts[1];
        
        let url = if method == "CONNECT" {
            // For CONNECT requests, URL is host:port
            format!("https://{}", url_part.split(':').next().unwrap_or(url_part))
        } else if url_part.starts_with("http") {
            url_part.to_string()
        } else {
            // Extract host from headers for relative URLs
            for line in &lines[1..] {
                if line.to_lowercase().starts_with("host:") {
                    let host = line[5..].trim();
                    return Some((method, format!("http://{}{}", host, url_part)));
                }
            }
            return None;
        };
        
        Some((method, url))
    } else {
        None
    }
}

async fn handle_connect(stream: &mut TcpStream, url: &str) {
    // Extract host from URL
    let host = url.replace("https://", "").replace("http://", "");
    let host_port = if host.contains(':') {
        host
    } else {
        format!("{}:443", host)
    };
    
    // Try to connect to the target server
    match TcpStream::connect(&host_port).await {
        Ok(_target) => {
            // Send 200 Connection Established
            let response = "HTTP/1.1 200 Connection Established\r\n\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
            
            // Note: In a full implementation, you'd tunnel data between client and target
            // For this demo, we just acknowledge the connection
        }
        Err(_) => {
            // Send connection failed
            let response = "HTTP/1.1 502 Bad Gateway\r\n\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
        }
    }
}

fn create_blocked_response(url: &str) -> String {
    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>🚫 Ad Blocked</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
            text-align: center; 
            padding: 20px; 
            background: #f5f5f5;
            color: #333;
        }}
        .container {{ 
            background: white; 
            padding: 30px; 
            border-radius: 10px; 
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            max-width: 400px;
            margin: 0 auto;
        }}
        .emoji {{ font-size: 48px; margin-bottom: 20px; }}
        h1 {{ color: #e74c3c; margin: 0 0 10px 0; }}
        .url {{ 
            background: #f8f9fa; 
            padding: 10px; 
            border-radius: 5px; 
            word-break: break-all;
            font-size: 12px;
            margin: 15px 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="emoji">🛡️</div>
        <h1>Ad Blocked!</h1>
        <p>This request was blocked by your ad blocker.</p>
        <div class="url">{}</div>
        <p><small>Protected by Rust Ad Blocker</small></p>
    </div>
</body>
</html>
"#, url);

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        html.len(),
        html
    )
}

fn create_allowed_response(url: &str) -> String {
    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>✅ Proxy Working</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
            text-align: center; 
            padding: 20px; 
            background: #f0f8ff;
            color: #333;
        }}
        .container {{ 
            background: white; 
            padding: 30px; 
            border-radius: 10px; 
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            max-width: 400px;
            margin: 0 auto;
        }}
        .emoji {{ font-size: 48px; margin-bottom: 20px; }}
        h1 {{ color: #27ae60; margin: 0 0 10px 0; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="emoji">✅</div>
        <h1>Proxy Working!</h1>
        <p>Your ad blocker proxy is working correctly.</p>
        <p><small>URL: {}</small></p>
        <p><em>Note: This is a demo response. In a full proxy, you'd see the actual website.</em></p>
    </div>
</body>
</html>
"#, url);

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        html.len(),
        html
    )
}

fn get_local_ip() -> Option<String> {
    use std::net::TcpStream;
    
    // Try to connect to a remote address to determine local IP
    if let Ok(stream) = TcpStream::connect("8.8.8.8:80") {
        if let Ok(local_addr) = stream.local_addr() {
            return Some(local_addr.ip().to_string());
        }
    }
    
    None
}