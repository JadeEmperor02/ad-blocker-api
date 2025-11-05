# 🚀 AWS Deployment Guide

## **Step 1: Launch EC2 Instance**

### Via AWS Console:
1. **Go to EC2 Dashboard** → Launch Instance
2. **Choose AMI**: Ubuntu Server 22.04 LTS (Free tier eligible)
3. **Instance Type**: t2.micro (Free tier - 1GB RAM)
4. **Key Pair**: Create new or use existing SSH key
5. **Security Group**: Create new with these rules:
   - **SSH**: Port 22, Source: Your IP
   - **DNS**: Port 53 UDP, Source: 0.0.0.0/0
   - **HTTP**: Port 80, Source: 0.0.0.0/0 (optional, for stats)
6. **Storage**: 8GB (free tier)
7. **Launch Instance**

### Via AWS CLI:
```bash
# Create security group
aws ec2 create-security-group \
    --group-name ad-blocker-sg \
    --description "Ad Blocker DNS Server"

# Add rules
aws ec2 authorize-security-group-ingress \
    --group-name ad-blocker-sg \
    --protocol tcp --port 22 --cidr 0.0.0.0/0

aws ec2 authorize-security-group-ingress \
    --group-name ad-blocker-sg \
    --protocol udp --port 53 --cidr 0.0.0.0/0

# Launch instance
aws ec2 run-instances \
    --image-id ami-0c02fb55956c7d316 \
    --count 1 \
    --instance-type t2.micro \
    --key-name your-key-pair \
    --security-groups ad-blocker-sg
```

---

## **Step 2: Connect to Your Instance**

```bash
# Get your instance public IP from AWS console
ssh -i your-key.pem ubuntu@your-instance-ip

# Example:
ssh -i ~/.ssh/my-key.pem ubuntu@3.15.123.456
```

---

## **Step 3: Setup the Server**

### Quick Setup Script:
```bash
# Run this on your EC2 instance
curl -sSL https://raw.githubusercontent.com/yourusername/ad-blocker-api/main/aws-setup.sh | bash
```

### Manual Setup:
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Install dependencies
sudo apt install -y build-essential pkg-config libssl-dev git

# Clone your project
git clone https://github.com/yourusername/ad-blocker-api.git
cd ad-blocker-api

# Build the VPN server
cargo build --release --example vpn_server

# Test run (Ctrl+C to stop)
sudo ./target/release/examples/vpn_server
```

---

## **Step 4: Create Systemd Service**

```bash
# Create service file
sudo tee /etc/systemd/system/ad-blocker-dns.service > /dev/null << 'EOF'
[Unit]
Description=Ad Blocker DNS Server
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=/home/ubuntu/ad-blocker-api
ExecStart=/home/ubuntu/ad-blocker-api/target/release/examples/vpn_server
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and start service
sudo systemctl daemon-reload
sudo systemctl enable ad-blocker-dns
sudo systemctl start ad-blocker-dns

# Check status
sudo systemctl status ad-blocker-dns

# View logs
sudo journalctl -u ad-blocker-dns -f
```

---

## **Step 5: Configure Your Phone**

### Get Your Server IP:
```bash
# From AWS console or:
curl -s https://api.ipify.org
```

### iPhone Setup:
1. **Settings** → **WiFi** → **(i)** next to your network
2. **Configure DNS** → **Manual**
3. **Remove** existing DNS servers
4. **Add DNS Server**: `your-ec2-ip`
5. **Save**

### Android Setup:
1. **Settings** → **WiFi** → **Long press** your network
2. **Modify Network** → **Advanced Options**
3. **IP Settings** → **Static**
4. **DNS 1**: `your-ec2-ip`
5. **DNS 2**: `8.8.8.8` (backup)
6. **Save**

---

## **Step 6: Test It Works**

### From Your Computer:
```bash
# Test blocked domain
dig @your-ec2-ip doubleclick.net
# Should return 0.0.0.0

# Test allowed domain  
dig @your-ec2-ip google.com
# Should return real IP
```

### From Your Phone:
1. **Open browser** on your phone
2. **Visit any website** with ads
3. **Ads should be blocked!** 🎉
4. **Check AWS logs**: `sudo journalctl -u ad-blocker-dns -f`

---

## **Step 7: Monitor & Maintain**

### View Real-time Logs:
```bash
sudo journalctl -u ad-blocker-dns -f
```

### Check Service Status:
```bash
sudo systemctl status ad-blocker-dns
```

### Restart Service:
```bash
sudo systemctl restart ad-blocker-dns
```

### Update Blocklists:
```bash
# The service automatically updates StevenBlack hosts on startup
sudo systemctl restart ad-blocker-dns
```

---

## **Step 8: Optional Enhancements**

### Add Web Dashboard:
```bash
# Install nginx for stats dashboard
sudo apt install -y nginx

# Create simple stats page
sudo tee /var/www/html/index.html > /dev/null << 'EOF'
<!DOCTYPE html>
<html>
<head><title>Ad Blocker Stats</title></head>
<body>
    <h1>🛡️ Ad Blocker DNS Server</h1>
    <p>Your DNS server is running!</p>
    <p>Check logs: <code>sudo journalctl -u ad-blocker-dns -f</code></p>
</body>
</html>
EOF

# Access at: http://your-ec2-ip
```

### Set Up Automatic Updates:
```bash
# Create update script
sudo tee /usr/local/bin/update-ad-blocker.sh > /dev/null << 'EOF'
#!/bin/bash
cd /home/ubuntu/ad-blocker-api
git pull
cargo build --release --example vpn_server
systemctl restart ad-blocker-dns
EOF

sudo chmod +x /usr/local/bin/update-ad-blocker.sh

# Add to crontab (update weekly)
echo "0 2 * * 0 /usr/local/bin/update-ad-blocker.sh" | sudo crontab -
```

---

## **💰 AWS Costs**

### Free Tier (First 12 months):
- **EC2 t2.micro**: 750 hours/month (FREE)
- **EBS Storage**: 30GB (FREE)
- **Data Transfer**: 15GB out (FREE)
- **Total**: $0/month for first year

### After Free Tier:
- **EC2 t2.micro**: ~$8.50/month
- **EBS Storage**: ~$2.40/month (20GB)
- **Data Transfer**: ~$0.09/GB
- **Total**: ~$11/month

---

## **🔒 Security Best Practices**

### Restrict SSH Access:
```bash
# Only allow your IP
aws ec2 authorize-security-group-ingress \
    --group-name ad-blocker-sg \
    --protocol tcp --port 22 --cidr YOUR-IP/32
```

### Enable CloudWatch Monitoring:
```bash
# Install CloudWatch agent
wget https://s3.amazonaws.com/amazoncloudwatch-agent/amazon_linux/amd64/latest/amazon-cloudwatch-agent.rpm
sudo rpm -U ./amazon-cloudwatch-agent.rpm
```

### Set Up Backup:
```bash
# Create AMI snapshot weekly
aws ec2 create-image \
    --instance-id i-1234567890abcdef0 \
    --name "ad-blocker-backup-$(date +%Y%m%d)"
```

---

## **🚀 Quick Deploy Script**

Save this as `aws-setup.sh`:

```bash
#!/bin/bash
set -e

echo "🚀 Setting up Ad Blocker DNS Server on AWS..."

# Update system
sudo apt update && sudo apt upgrade -y

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Install dependencies
sudo apt install -y build-essential pkg-config libssl-dev git

# Clone project (replace with your repo)
git clone https://github.com/yourusername/ad-blocker-api.git
cd ad-blocker-api

# Build
cargo build --release --example vpn_server

# Create service
sudo tee /etc/systemd/system/ad-blocker-dns.service > /dev/null << 'EOF'
[Unit]
Description=Ad Blocker DNS Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/home/ubuntu/ad-blocker-api
ExecStart=/home/ubuntu/ad-blocker-api/target/release/examples/vpn_server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Start service
sudo systemctl daemon-reload
sudo systemctl enable ad-blocker-dns
sudo systemctl start ad-blocker-dns

echo "✅ Ad Blocker DNS Server is running!"
echo "📍 Your DNS server IP: $(curl -s https://api.ipify.org)"
echo "📱 Configure your phone to use this IP as DNS server"
echo "📊 Check status: sudo systemctl status ad-blocker-dns"
echo "📋 View logs: sudo journalctl -u ad-blocker-dns -f"
```

Run with:
```bash
curl -sSL your-script-url | bash
```

---

## **🎉 You're Done!**

Your phone now has **global ad blocking** that works on:
- ✅ **Any WiFi network**
- ✅ **Cellular data**  
- ✅ **Hotel WiFi**
- ✅ **Public WiFi**
- ✅ **Anywhere in the world!**

**Total setup time**: ~15 minutes  
**Monthly cost**: Free for 12 months, then ~$11/month  
**Domains blocked**: 100,000+  

Your AWS-hosted ad blocker is ready! 🌍📱🛡️