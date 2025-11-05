use ad_blocker_api::prelude::*;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Mobile proxy server that your phone can connect to
#[tokio::main]
async fn main() -> Result<()> {
    println!("📱 Mobile Ad-Blocking Proxy Server");
    println!("==================================");
    
    // Create ad blocker
    let blocker = SimpleAdBlocker::new().await?;
    
    // Get local IP address
    let local_ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    let port = 8888;
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    
    println!("🌐 Starting proxy server...");
    println!("📍 Server address: http://{}:{}", local_ip, port);
    println!();
    println!("📱 PHONE SETUP INSTRUCTIONS:");
    println!("1. Connect your phone to the same WiFi network");
    println!("2. Go to WiFi settings on your phone");
    println!("3. Configure HTTP Proxy:");
    println!("   • Server: {}", local_ip);
    println!("   • Port: {}", port);
    println!("4. Save and browse - ads will be blocked!");
    println!();
    println!("🔧 Test URLs to try on your phone:");
    println!("   • https://www.github.com (should work)");
    println!("   • https://www.google.com (should work)");
    println!("   • Any website with ads (ads blocked!)");
    println!();
    
    let listener = TcpListener::bind(addr).await?;
    println!("✅ Proxy server running! Press Ctrl+C to stop.");
    
    let mut request_count = 0u64;
    let mut blocked_count = 0u64;
    
    let mut request_count = 0u64;
    
    loop {
        let (stream, _) = listener.accept().await?;
        request_count += 1;
        
        // Handle connection directly (simplified for demo)
        if let Err(e) = handle_connection(stream, &blocker, request_count).await {
            eprintln!("Connection #{} error: {}", request_count, e);
        }
    }
}

async fn handle_connection(mut stream: TcpStream, blocker: &SimpleAdBlocker, req_num: u64) -> Result<()> {
    let mut buffer = [0; 8192];
    let n = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);
    
    let lines: Vec<&str> = request.lines().collect();
    if lines.is_empty() {
        return Ok(());
    }
    
    let first_line = lines[0];
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return Ok(());
    }
    
    let method = parts[0];
    let url_part = parts[1];
    
    // Handle CONNECT method (HTTPS tunneling)
    if method == "CONNECT" {
        let host_port = url_part;
        let url = format!("https://{}", host_port.split(':').next().unwrap_or(host_port));
        
        println!("📱 Request #{}: {} (HTTPS)", req_num, url);
        
        // Check if should be blocked
        let block_result = blocker.check_url(&url).await?;
        
        if block_result.should_block {
            println!("   🚫 BLOCKED: {} ({:?})", block_result.reason, block_result.category);
            
            // Send connection refused
            let response = "HTTP/1.1 403 Forbidden\r\n\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
            return Ok(());
        }
        
        println!("   ✅ ALLOWED: Tunneling HTTPS connection");
        
        // Establish tunnel to target server
        match TcpStream::connect(host_port).await {
            Ok(target) => {
                // Send 200 Connection Established
                let response = "HTTP/1.1 200 Connection Established\r\n\r\n";
                stream.write_all(response.as_bytes()).await?;
                
                // Start bidirectional copying
                let (mut client_read, mut client_write) = stream.into_split();
                let (mut target_read, mut target_write) = target.into_split();
                
                let client_to_target = tokio::io::copy(&mut client_read, &mut target_write);
                let target_to_client = tokio::io::copy(&mut target_read, &mut client_write);
                
                // Wait for either direction to close
                tokio::select! {
                    _ = client_to_target => {},
                    _ = target_to_client => {},
                }
            }
            Err(e) => {
                println!("   ❌ Failed to connect to {}: {}", host_port, e);
                let response = "HTTP/1.1 502 Bad Gateway\r\n\r\n";
                let _ = stream.write_all(response.as_bytes()).await;
            }
        }
    } else {
        // Handle regular HTTP requests
        let url = if url_part.starts_with("http") {
            url_part.to_string()
        } else {
            // Extract host from headers
            let mut host = "";
            for line in &lines[1..] {
                if line.to_lowercase().starts_with("host:") {
                    host = line[5..].trim();
                    break;
                }
            }
            if host.is_empty() {
                return Ok(());
            }
            format!("http://{}{}", host, url_part)
        };
        
        println!("📱 Request #{}: {} (HTTP)", req_num, url);
        
        // Check if should be blocked
        let block_result = blocker.check_url(&url).await?;
        
        if block_result.should_block {
            println!("   🚫 BLOCKED: {} ({:?})", block_result.reason, block_result.category);
            
            // Send blocked response
            let response = create_blocked_response(&url, &block_result.reason);
            let _ = stream.write_all(response.as_bytes()).await;
            return Ok(());
        }
        
        println!("   ✅ ALLOWED: Forwarding HTTP request");
        
        // Forward the request (simplified - just return a basic response)
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Request Allowed</h1><p>This is a simplified proxy response. In production, this would forward to the actual server.</p></body></html>";
        let _ = stream.write_all(response.as_bytes()).await;
    }
    
    Ok(())
}

fn create_blocked_response(url: &str, reason: &str) -> String {
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
            padding: 50px; 
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
        .reason {{ color: #666; font-size: 14px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="emoji">🛡️</div>
        <h1>Ad Blocked!</h1>
        <p class="reason">{}</p>
        <div class="url">{}</div>
        <p><small>Protected by Rust Ad Blocker</small></p>
    </div>
</body>
</html>
"#, reason, url);

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