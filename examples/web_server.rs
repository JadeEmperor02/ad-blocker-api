use ad_blocker_api::prelude::*;
use anyhow::Result;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Simple HTTP server that provides ad-blocking as a service
#[tokio::main]
async fn main() -> Result<()> {
    println!("🌐 Starting Ad Blocker Web Service on http://localhost:8080");
    
    // Create a single blocker instance
    let blocker = SimpleAdBlocker::new().await?;
    
    // Start server
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    println!("📡 Server ready! Try these endpoints:");
    println!("  GET  /check?url=https://example.com");
    println!("  GET  /stats");
    println!("  GET  / (web interface)");
    
    loop {
        let (mut stream, _) = listener.accept().await?;
        
        // Handle requests sequentially to avoid threading issues
        let mut buffer = [0; 1024];
        if let Ok(n) = stream.read(&mut buffer).await {
            let request = String::from_utf8_lossy(&buffer[..n]);
            let response = handle_request(&request, &blocker).await;
            let _ = stream.write_all(response.as_bytes()).await;
        }
    }
}

async fn handle_request(request: &str, blocker: &SimpleAdBlocker) -> String {
    let lines: Vec<&str> = request.lines().collect();
    if lines.is_empty() {
        return http_response(400, "Bad Request");
    }
    
    let request_line = lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return http_response(400, "Bad Request");
    }
    
    let method = parts[0];
    let path = parts[1];
    
    match (method, path.split('?').next().unwrap_or(path)) {
        ("GET", "/check") => {
            if let Some(url) = extract_url_param(path) {
                match blocker.check_url(&url).await {
                    Ok(result) => {
                        let json = json!({
                            "url": url,
                            "blocked": result.should_block,
                            "reason": result.reason,
                            "category": result.category
                        });
                        http_json_response(200, &json.to_string())
                    }
                    Err(_) => http_response(500, "Internal Server Error")
                }
            } else {
                http_response(400, "Missing url parameter")
            }
        }
        ("GET", "/stats") => {
            let stats = blocker.get_stats().await;
            let json = json!({
                "total_requests": stats.total_requests,
                "blocked_requests": stats.blocked_requests,
                "ads_blocked": stats.ads_blocked,
                "trackers_blocked": stats.trackers_blocked,
                "block_percentage": stats.block_percentage()
            });
            http_json_response(200, &json.to_string())
        }
        ("GET", "/") => {
            let html = r#"
<!DOCTYPE html>
<html>
<head><title>Ad Blocker API</title></head>
<body>
    <h1>🛡️ Ad Blocker API</h1>
    <h2>Test URL:</h2>
    <input type="text" id="url" placeholder="https://example.com" style="width: 300px;">
    <button onclick="checkUrl()">Check</button>
    <div id="result"></div>
    
    <script>
    async function checkUrl() {
        const url = document.getElementById('url').value;
        const response = await fetch(`/check?url=${encodeURIComponent(url)}`);
        const data = await response.json();
        document.getElementById('result').innerHTML = 
            `<h3>Result:</h3>
             <p><strong>Blocked:</strong> ${data.blocked ? '🚫 Yes' : '✅ No'}</p>
             <p><strong>Reason:</strong> ${data.reason}</p>
             <p><strong>Category:</strong> ${data.category}</p>`;
    }
    </script>
</body>
</html>"#;
            http_html_response(200, html)
        }
        _ => http_response(404, "Not Found")
    }
}

fn extract_url_param(path: &str) -> Option<String> {
    if let Some(query) = path.split('?').nth(1) {
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "url" {
                    return Some(urlencoding::decode(value).ok()?.into_owned());
                }
            }
        }
    }
    None
}

fn http_response(status: u16, body: &str) -> String {
    format!(
        "HTTP/1.1 {} OK\r\nContent-Length: {}\r\n\r\n{}",
        status,
        body.len(),
        body
    )
}

fn http_json_response(status: u16, json: &str) -> String {
    format!(
        "HTTP/1.1 {} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        status,
        json.len(),
        json
    )
}

fn http_html_response(status: u16, html: &str) -> String {
    format!(
        "HTTP/1.1 {} OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        status,
        html.len(),
        html
    )
}