# 🌍 VPN-Style Ad Blocker Deployment

## 🚀 **Deploy to Cloud (Works Anywhere!)**

### **Option 1: DigitalOcean Droplet (Recommended)**

#### Step 1: Create Droplet
```bash
# Create a $5/month droplet
# Ubuntu 22.04, Basic plan, 1GB RAM
# Choose datacenter closest to you
```

#### Step 2: Setup Server
```bash
# SSH into your droplet
ssh root@your-droplet-ip

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install dependencies
apt update
apt install -y build-essential pkg-config libssl-dev

# Clone your project (or upload files)
git clone https://github.com/yourusername/ad-blocker-api.git
cd ad-blocker-api

# Build the VPN server
cargo build --release --example vpn_server

# Run the server (as root for port 53)
sudo ./target/release/examples/vpn_server
```

#### Step 3: Configure Firewall
```bash
# Allow DNS traffic
ufw allow 53/udp
ufw allow ssh
ufw enable
```

#### Step 4: Setup as Service
```bash
# Create systemd service
sudo nano /etc/systemd/system/ad-blocker-vpn.service
```

```ini
[Unit]
Description=Ad Blocker VPN DNS Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/ad-blocker-api
ExecStart=/root/ad-blocker-api/target/release/examples/vpn_server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start service
sudo systemctl enable ad-blocker-vpn
sudo systemctl start ad-blocker-vpn
sudo systemctl status ad-blocker-vpn
```

---

### **Option 2: AWS EC2 (Free Tier)**

#### Step 1: Launch EC2 Instance
- AMI: Ubuntu Server 22.04 LTS
- Instance Type: t2.micro (free tier)
- Security Group: Allow UDP port 53, SSH port 22

#### Step 2: Setup (Same as DigitalOcean)
```bash
# SSH and setup Rust + dependencies
# Build and run the VPN server
```

---

### **Option 3: Google Cloud Platform**

#### Step 1: Create VM Instance
```bash
# Create f1-micro instance (free tier)
gcloud compute instances create ad-blocker-vpn \
    --zone=us-central1-a \
    --machine-type=f1-micro \
    --image-family=ubuntu-2204-lts \
    --image-project=ubuntu-os-cloud
```

#### Step 2: Configure Firewall
```bash
# Allow DNS traffic
gcloud compute firewall-rules create allow-dns \
    --allow udp:53 \
    --source-ranges 0.0.0.0/0
```

---

## 📱 **Phone Setup (Works Anywhere!)**

Once deployed, your phone can use this DNS server on **any network**:

### **iPhone:**
1. Settings → General → VPN & Device Management
2. Add VPN Configuration → Type: IKEv2
3. Or use DNS: Settings → WiFi → Configure DNS → Manual
4. Add your server IP: `your-server-ip`

### **Android:**
1. Settings → Network & Internet → Advanced → Private DNS
2. Private DNS provider hostname: `your-server-ip`
3. Or WiFi → Modify Network → Advanced → DNS

### **Global DNS Apps:**
- **1.1.1.1 app** (Cloudflare) - can set custom DNS
- **DNS Changer** apps - point to your server
- **AdGuard** - can use custom DNS servers

---

## 🔧 **Advanced Features**

### **Add Authentication**
```rust
// Add API key authentication
if !request.headers().get("x-api-key").map_or(false, |key| key == "your-secret-key") {
    return Err(anyhow::anyhow!("Unauthorized"));
}
```

### **Add HTTPS DNS (DoH)**
```rust
// Support DNS over HTTPS for better security
// Use hyper to handle HTTPS requests on port 443
```

### **Add Metrics Dashboard**
```rust
// Create web dashboard to monitor blocking stats
// Show blocked domains, request counts, etc.
```

### **Custom Blocklists**
```rust
// Allow users to add custom domains via API
blocker.add_blocked_domain("custom-ad-domain.com").await;
```

---

## 💰 **Cost Breakdown**

### **DigitalOcean:**
- Basic Droplet: $5/month
- Bandwidth: 1TB included
- **Total: ~$5/month**

### **AWS EC2:**
- t2.micro: Free for 12 months
- After free tier: ~$8/month
- **Total: Free first year, then ~$8/month**

### **Google Cloud:**
- f1-micro: Always free
- Network egress: First 1GB free/month
- **Total: Nearly free**

---

## 🌟 **Benefits of VPN-Style DNS Blocking**

✅ **Works on ANY network** (WiFi, cellular, ethernet)  
✅ **Blocks ads system-wide** (all apps, browsers)  
✅ **Faster browsing** (no ad downloads)  
✅ **Better privacy** (no tracking scripts)  
✅ **Saves bandwidth** (especially on cellular)  
✅ **Works on all devices** (phone, laptop, tablet)  
✅ **No app installation** required  

---

## 🚀 **Quick Deploy Script**

```bash
#!/bin/bash
# deploy.sh - Quick deployment script

# Update system
apt update && apt upgrade -y

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Install dependencies
apt install -y build-essential pkg-config libssl-dev git

# Clone and build
git clone https://github.com/yourusername/ad-blocker-api.git
cd ad-blocker-api
cargo build --release --example vpn_server

# Setup firewall
ufw allow 53/udp
ufw allow ssh
ufw --force enable

# Create service
cat > /etc/systemd/system/ad-blocker-vpn.service << EOF
[Unit]
Description=Ad Blocker VPN DNS Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$(pwd)
ExecStart=$(pwd)/target/release/examples/vpn_server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Start service
systemctl enable ad-blocker-vpn
systemctl start ad-blocker-vpn

echo "🎉 Ad Blocker VPN DNS Server deployed!"
echo "📍 Your DNS server: $(curl -s https://api.ipify.org)"
echo "📱 Configure your phone to use this IP as DNS server"
```

Run with: `curl -sSL your-script-url | sudo bash`

---

## 🔒 **Security Considerations**

- **Rate limiting**: Prevent DNS amplification attacks
- **Authentication**: Add API keys for management
- **Monitoring**: Log suspicious activity
- **Updates**: Keep blocklists updated automatically
- **Backup**: Regular configuration backups

Your phone now has **global ad blocking** that works anywhere! 🌍📱🛡️