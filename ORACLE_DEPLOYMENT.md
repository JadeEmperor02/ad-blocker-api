# 🚀 Oracle Cloud Deployment Guide

## **Why Oracle Cloud is Perfect for This:**
- ✅ **Always Free Tier** (not just 12 months like AWS)
- ✅ **2 AMD Compute instances** (1/8 OCPU, 1GB RAM each)
- ✅ **10GB Block Storage** 
- ✅ **10TB Outbound Data Transfer** per month
- ✅ **No credit card expiration worries**

---

## **Step 1: Create Oracle Cloud Account**

1. Go to [oracle.com/cloud/free](https://oracle.com/cloud/free)
2. Sign up for **Always Free** account
3. Complete verification (may take a few minutes)
4. Login to **Oracle Cloud Console**

---

## **Step 2: Launch Compute Instance**

### Via Oracle Cloud Console:
1. **Navigate**: Compute → Instances → Create Instance
2. **Name**: `ad-blocker-dns`
3. **Placement**: Keep default availability domain
4. **Image**: 
   - Click "Change Image"
   - Select **Ubuntu** → **22.04** (Canonical)
   - Click "Select Image"
5. **Shape**: 
   - Click "Change Shape"
   - Select **VM.Standard.E2.1.Micro** (Always Free)
   - 1/8 OCPU, 1GB RAM
6. **Networking**:
   - Keep **Create new virtual cloud network**
   - Keep **Create new public subnet**
   - ✅ **Assign a public IPv4 address**
7. **SSH Keys**:
   - **Generate SSH key pair** (download both keys)
   - Or upload your existing public key
8. **Boot Volume**: Keep defaults (Always Free eligible)
9. **Click "Create"**

### Wait for Instance to Start:
- Status will change from "PROVISIONING" → "RUNNING"
- Note your **Public IP Address**

---

## **Step 3: Configure Network Security**

### Open Required Ports:
1. **Go to**: Networking → Virtual Cloud Networks
2. **Click** your VCN name
3. **Click** "Public Subnet" 
4. **Click** "Default Security List"
5. **Add Ingress Rules**:

**Rule 1 - SSH:**
- Source CIDR: `0.0.0.0/0`
- IP Protocol: `TCP`
- Destination Port Range: `22`

**Rule 2 - DNS:**
- Source CIDR: `0.0.0.0/0` 
- IP Protocol: `UDP`
- Destination Port Range: `53`

**Rule 3 - HTTP (Optional):**
- Source CIDR: `0.0.0.0/0`
- IP Protocol: `TCP` 
- Destination Port Range: `80`

6. **Click "Add Ingress Rules"**

---

## **Step 4: Connect to Your Instance**

### From Windows (PowerShell):
```powershell
# If you generated keys, use the private key file
ssh -i path\to\your-private-key ubuntu@YOUR-PUBLIC-IP

# Example:
ssh -i C:\Users\YourName\Downloads\ssh-key-2024-11-05.key ubuntu@129.213.123.456
```

### From Mac/Linux:
```bash
# Make key file secure
chmod 600 ~/Downloads/ssh-key-2024-11-05.key

# Connect
ssh -i ~/Downloads/ssh-key-2024-11-05.key ubuntu@YOUR-PUBLIC-IP
```

---

## **Step 5: Install Ad Blocker DNS Server**

### Quick Setup Script:
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install dnsmasq (lightweight DNS server)
sudo apt install -y dnsmasq curl

# Stop systemd-resolved (conflicts with dnsmasq)
sudo systemctl stop systemd-resolved
sudo systemctl disable systemd-resolved

# Configure dnsmasq
sudo tee /etc/dnsmasq.conf > /dev/null << 'EOF'
# Listen on all interfaces
port=53
listen-address=0.0.0.0
bind-interfaces

# Upstream DNS servers
server=8.8.8.8
server=1.1.1.1
server=1.0.0.1

# Use custom hosts file for blocking
addn-hosts=/etc/hosts.blocked

# Cache settings
cache-size=10000
neg-ttl=3600

# Logging (optional)
log-queries
log-facility=/var/log/dnsmasq.log
EOF

# Download StevenBlack hosts file (100,000+ blocked domains)
echo "📥 Downloading ad blocking hosts file..."
sudo curl -o /etc/hosts.blocked https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts

# Create systemd service
sudo tee /etc/systemd/system/ad-blocker-dns.service > /dev/null << 'EOF'
[Unit]
Description=Ad Blocker DNS Server
After=network.target
Wants=network.target

[Service]
Type=forking
User=root
Group=root
ExecStart=/usr/sbin/dnsmasq
ExecReload=/bin/kill -HUP $MAINPID
PIDFile=/var/run/dnsmasq.pid
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Start and enable the service
sudo systemctl daemon-reload
sudo systemctl enable ad-blocker-dns
sudo systemctl start ad-blocker-dns

# Check status
sudo systemctl status ad-blocker-dns
```

---

## **Step 6: Configure Oracle Cloud Firewall**

### Ubuntu Firewall (UFW):
```bash
# Enable firewall
sudo ufw enable

# Allow SSH
sudo ufw allow 22/tcp

# Allow DNS
sudo ufw allow 53/udp

# Allow HTTP (optional, for stats)
sudo ufw allow 80/tcp

# Check status
sudo ufw status
```

---

## **Step 7: Test Your DNS Server**

### From Your Computer:
```bash
# Test blocked domain (should return 0.0.0.0)
nslookup doubleclick.net YOUR-ORACLE-IP

# Test allowed domain (should return real IP)
nslookup google.com YOUR-ORACLE-IP

# Windows PowerShell:
Resolve-DnsName -Name doubleclick.net -Server YOUR-ORACLE-IP
Resolve-DnsName -Name google.com -Server YOUR-ORACLE-IP
```

---

## **Step 8: Configure Your Phone**

### Get Your Oracle Instance IP:
```bash
# From your Oracle instance
curl -s https://api.ipify.org
```

### iPhone Setup:
1. **Settings** → **WiFi** → **(i)** next to your network
2. **Configure DNS** → **Manual**
3. **Remove** existing DNS servers
4. **Add DNS Server**: `your-oracle-ip`
5. **Save**

### Android Setup:
1. **Settings** → **WiFi** → **Long press** your network
2. **Modify Network** → **Advanced Options**
3. **IP Settings** → **Static**
4. **DNS 1**: `your-oracle-ip`
5. **DNS 2**: `8.8.8.8` (backup)
6. **Save**

---

## **Step 9: Monitor and Maintain**

### View Real-time DNS Queries:
```bash
# Watch live DNS queries
sudo tail -f /var/log/dnsmasq.log

# Check service status
sudo systemctl status ad-blocker-dns

# Restart service
sudo systemctl restart ad-blocker-dns
```

### Update Blocklist (Weekly):
```bash
# Create update script
sudo tee /usr/local/bin/update-blocklist.sh > /dev/null << 'EOF'
#!/bin/bash
echo "Updating ad blocking hosts..."
curl -o /tmp/hosts.new https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts
if [ $? -eq 0 ]; then
    sudo mv /tmp/hosts.new /etc/hosts.blocked
    sudo systemctl restart ad-blocker-dns
    echo "Blocklist updated successfully!"
else
    echo "Failed to download new blocklist"
fi
EOF

sudo chmod +x /usr/local/bin/update-blocklist.sh

# Add to crontab (update every Sunday at 2 AM)
echo "0 2 * * 0 /usr/local/bin/update-blocklist.sh" | sudo crontab -
```

---

## **Step 10: Optional Enhancements**

### Add Web Dashboard:
```bash
# Install nginx
sudo apt install -y nginx

# Create simple stats page
sudo tee /var/www/html/index.html > /dev/null << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>🛡️ Ad Blocker DNS Stats</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
        .container { background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }
        .stat-box { background: #f8f9fa; padding: 20px; border-radius: 8px; text-align: center; }
        .stat-number { font-size: 2em; font-weight: bold; color: #e74c3c; }
        pre { background: #f8f9fa; padding: 15px; border-radius: 5px; overflow-x: auto; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🛡️ Ad Blocker DNS Server</h1>
        <p>Your DNS server is running and blocking ads!</p>
        
        <div class="stats">
            <div class="stat-box">
                <div class="stat-number">100,000+</div>
                <div>Blocked Domains</div>
            </div>
            <div class="stat-box">
                <div class="stat-number">24/7</div>
                <div>Protection</div>
            </div>
            <div class="stat-box">
                <div class="stat-number">FREE</div>
                <div>Forever</div>
            </div>
        </div>
        
        <h3>📱 Phone Setup:</h3>
        <pre>DNS Server: <span id="server-ip">Loading...</span>
Port: 53</pre>
        
        <h3>🔧 Test Commands:</h3>
        <pre>nslookup doubleclick.net <span id="server-ip2">your-ip</span>
nslookup google.com <span id="server-ip3">your-ip</span></pre>
        
        <p><small>Powered by Oracle Cloud Always Free Tier</small></p>
    </div>
    
    <script>
        fetch('https://api.ipify.org?format=json')
            .then(response => response.json())
            .then(data => {
                document.getElementById('server-ip').textContent = data.ip;
                document.getElementById('server-ip2').textContent = data.ip;
                document.getElementById('server-ip3').textContent = data.ip;
            });
    </script>
</body>
</html>
EOF

# Start nginx
sudo systemctl enable nginx
sudo systemctl start nginx

# Access at: http://your-oracle-ip
```

---

## **💰 Oracle Cloud Costs**

### Always Free Tier Includes:
- ✅ **2 Compute instances** (VM.Standard.E2.1.Micro)
- ✅ **1/8 OCPU and 1GB RAM** each
- ✅ **2 Block Volumes** (100GB total)
- ✅ **10TB Outbound Data Transfer** per month
- ✅ **No time limit** (unlike AWS 12-month free tier)

### Your DNS Server Uses:
- **1 Compute instance** (well within limits)
- **~5GB storage** (well within 100GB limit)
- **Minimal bandwidth** (DNS queries are tiny)

**Total Cost: $0/month forever!** 🎉

---

## **🔒 Security Best Practices**

### Secure SSH Access:
```bash
# Change SSH port (optional)
sudo sed -i 's/#Port 22/Port 2222/' /etc/ssh/sshd_config
sudo systemctl restart ssh

# Disable password authentication (key-only)
sudo sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
sudo systemctl restart ssh
```

### Enable Automatic Updates:
```bash
# Install unattended upgrades
sudo apt install -y unattended-upgrades

# Configure automatic security updates
sudo dpkg-reconfigure -plow unattended-upgrades
```

---

## **🚀 Quick Deploy Script**

Save this as `oracle-setup.sh` and run on your instance:

```bash
#!/bin/bash
set -e

echo "🚀 Setting up Ad Blocker DNS Server on Oracle Cloud..."

# Update system
sudo apt update && sudo apt upgrade -y

# Install dnsmasq
sudo apt install -y dnsmasq curl nginx

# Stop conflicting service
sudo systemctl stop systemd-resolved
sudo systemctl disable systemd-resolved

# Configure dnsmasq
sudo tee /etc/dnsmasq.conf > /dev/null << 'EOF'
port=53
listen-address=0.0.0.0
bind-interfaces
server=8.8.8.8
server=1.1.1.1
addn-hosts=/etc/hosts.blocked
cache-size=10000
log-queries
log-facility=/var/log/dnsmasq.log
EOF

# Download blocklist
echo "📥 Downloading ad blocking hosts..."
sudo curl -o /etc/hosts.blocked https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts

# Start services
sudo systemctl enable dnsmasq nginx
sudo systemctl start dnsmasq nginx

# Configure firewall
sudo ufw --force enable
sudo ufw allow 22/tcp
sudo ufw allow 53/udp
sudo ufw allow 80/tcp

# Get public IP
PUBLIC_IP=$(curl -s https://api.ipify.org)

echo "✅ Ad Blocker DNS Server is running!"
echo "📍 Your DNS server IP: $PUBLIC_IP"
echo "📱 Configure your phone to use this IP as DNS server"
echo "🌐 Web dashboard: http://$PUBLIC_IP"
echo "📊 Check logs: sudo tail -f /var/log/dnsmasq.log"
```

Run with:
```bash
curl -sSL your-script-url | bash
```

---

## **🎉 You're Done!**

Your Oracle Cloud ad blocker provides:
- ✅ **Global ad blocking** on any network
- ✅ **100,000+ blocked domains**
- ✅ **Free forever** (Oracle Always Free)
- ✅ **Works on all devices** that use your DNS
- ✅ **No bandwidth limits** for DNS queries

Configure your phone's DNS to your Oracle IP and enjoy ad-free browsing everywhere! 🌍📱🛡️