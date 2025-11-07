#!/bin/bash

echo "🚀 Deploying Enhanced Dynamic Ad Blocker DNS Server"
echo "=================================================="

# Stop existing DNS server if running
echo "🛑 Stopping existing DNS server..."
sudo pkill -f ultimate_dns || echo "No existing server found"

# Update dependencies
echo "📦 Updating dependencies..."
cargo update

# Add regex dependency if not present
if ! grep -q "regex" Cargo.toml; then
    echo "➕ Adding regex dependency..."
    echo 'regex = "1.0"' >> Cargo.toml
fi

# Build the enhanced DNS server
echo "🔨 Building enhanced DNS server..."
cargo build --release --example ultimate_dns

# Create systemd service for auto-restart
echo "⚙️ Creating systemd service..."
sudo tee /etc/systemd/system/enhanced-dns.service > /dev/null <<EOF
[Unit]
Description=Enhanced Dynamic Ad Blocker DNS Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$(pwd)
ExecStart=$(pwd)/target/release/examples/ultimate_dns
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Enable and start the service
echo "🚀 Starting enhanced DNS service..."
sudo systemctl daemon-reload
sudo systemctl enable enhanced-dns
sudo systemctl start enhanced-dns

# Show status
echo "📊 Service status:"
sudo systemctl status enhanced-dns --no-pager -l

echo ""
echo "✅ Enhanced Dynamic Ad Blocker DNS Server deployed!"
echo "🌐 Server is now running with:"
echo "   • Enhanced dynamic ad detection"
echo "   • JavaScript ad injection blocking"
echo "   • Programmatic advertising detection"
echo "   • Real-time bidding platform blocking"
echo "   • Connection monitoring capabilities"
echo ""
echo "📱 Configure your devices to use this server's IP as DNS"
echo "🔧 Monitor logs with: sudo journalctl -u enhanced-dns -f"
echo "🛑 Stop with: sudo systemctl stop enhanced-dns"