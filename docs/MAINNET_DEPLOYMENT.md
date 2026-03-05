# YRUST Chain 主网部署完全指南

> 从零开始部署一个生产级的区块链主网

## 📋 部署前准备

### 1. 服务器要求

| 配置 | 最低要求 | 推荐配置 |
|------|----------|----------|
| CPU | 4 核 | 8 核+ |
| 内存 | 8 GB | 16 GB+ |
| 存储 | 100 GB SSD | 500 GB NVMe |
| 带宽 | 10 Mbps | 100 Mbps+ |
| 系统 | Ubuntu 22.04 LTS | Ubuntu 22.04 LTS |

### 2. 域名和证书

- 域名：`yrust.chain` 或你的域名
- SSL 证书：Let's Encrypt 免费证书

### 3. 安全设置

```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装必要工具
sudo apt install -y ufw fail2ban nginx certbot python3-certbot-nginx

# 配置防火墙
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 8000/tcp  # P2P
sudo ufw allow 30303/tcp # 备用 P2P
sudo ufw enable
```

## 🚀 部署步骤

### 步骤 1: 安装 Docker

```bash
# 安装 Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# 安装 Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# 添加用户到 docker 组
sudo usermod -aG docker $USER
newgrp docker
```

### 步骤 2: 克隆项目

```bash
# 创建目录
mkdir -p /opt/blockchain
cd /opt/blockchain

# 克隆代码
git clone https://github.com/yourusername/yrust-chain.git
cd yrust-chain

# 创建数据目录
mkdir -p data logs
```

### 步骤 3: 配置主网参数

创建 `config/mainnet.toml`:

```toml
[network]
network_id = "yrust-mainnet-1"
p2p_port = 8000
rpc_port = 3000
bootstrap_nodes = [
    "/dns4/bootstrap1.yrust.chain/tcp/8000",
    "/dns4/bootstrap2.yrust.chain/tcp/8000",
]

[consensus]
# POW 难度
difficulty = 6
# 区块奖励（每 4 年减半）
block_reward = 5000000000  # 50 YRU
# 减半周期
halving_period = 2100000   # 约 4 年

[genesis]
# 创世区块时间
timestamp = 1704067200
# 创世分配
genesis_allocations = [
    { address = "1GenesisFoundation...", amount = 2100000000000000 },  # 10% 基金会
    { address = "1GenesisTeam...", amount = 1050000000000000 },       # 5% 团队
    { address = "1GenesisCommunity...", amount = 525000000000000 },   # 2.5% 社区
]

[api]
# 是否启用 CORS
cors_enabled = true
# 速率限制
rate_limit = 1000  # 每分钟请求数

[storage]
# 数据目录
data_dir = "/opt/blockchain/data"
# 缓存大小
cache_size = 512  # MB
```

### 步骤 4: 配置 Nginx

```bash
sudo tee /etc/nginx/sites-available/yrust-chain << 'EOF'
upstream yrust_api {
    server 127.0.0.1:3000;
    keepalive 64;
}

# HTTP 重定向到 HTTPS
server {
    listen 80;
    server_name yrust.chain www.yrust.chain;
    
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }
    
    location / {
        return 301 https://$server_name$request_uri;
    }
}

# HTTPS 主站
server {
    listen 443 ssl http2;
    server_name yrust.chain www.yrust.chain;
    
    # SSL 证书
    ssl_certificate /etc/letsencrypt/live/yrust.chain/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yrust.chain/privkey.pem;
    ssl_trusted_certificate /etc/letsencrypt/live/yrust.chain/chain.pem;
    
    # SSL 配置
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;
    
    # 安全头
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    
    # Gzip
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css application/json application/javascript;
    
    # 前端静态文件
    location / {
        root /opt/blockchain/yrust-chain/frontend;
        index index.html;
        try_files $uri $uri/ /index.html;
        
        # 缓存
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }
    
    # API 代理
    location /api/ {
        proxy_pass http://yrust_api/;
        proxy_http_version 1.1;
        
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket 支持
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # 超时
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # 限流
        limit_req zone=api_limit burst=20 nodelay;
    }
    
    # 健康检查
    location /health {
        proxy_pass http://yrust_api/api/stats;
        access_log off;
    }
}
EOF

# 启用站点
sudo ln -s /etc/nginx/sites-available/yrust-chain /etc/nginx/sites-enabled/
sudo rm /etc/nginx/sites-enabled/default

# 创建限流区域
sudo tee -a /etc/nginx/nginx.conf << 'EOF'
http {
    # API 限流
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
}
EOF

# 测试配置
sudo nginx -t
sudo systemctl restart nginx
```

### 步骤 5: 申请 SSL 证书

```bash
# 使用 Certbot 申请证书
sudo certbot --nginx -d yrust.chain -d www.yrust.chain

# 自动续期
sudo systemctl enable certbot.timer
```

### 步骤 6: 构建和启动

```bash
# 构建 Docker 镜像
cd /opt/blockchain/yrust-chain
docker build -t yrust-chain:mainnet -f deploy/Dockerfile .

# 创建 docker-compose.yml
cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  yrust-node:
    image: yrust-chain:mainnet
    container_name: yrust-mainnet
    restart: unless-stopped
    ports:
      - "127.0.0.1:3000:3000"  # 仅本地访问，通过 Nginx 代理
      - "8000:8000"             # P2P 端口
      - "8000:8000/udp"         # QUIC
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
      - ./config:/app/config:ro
    environment:
      - RUST_LOG=info
      - NODE_ENV=production
      - CONFIG_FILE=/app/config/mainnet.toml
    command: ["node", "--config", "/app/config/mainnet.toml"]
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/stats"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    ulimits:
      nofile:
        soft: 65536
        hard: 65536
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "10"

  # 监控
  prometheus:
    image: prom/prometheus:latest
    container_name: yrust-prometheus
    restart: unless-stopped
    ports:
      - "127.0.0.1:9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
    profiles:
      - monitoring

  grafana:
    image: grafana/grafana:latest
    container_name: yrust-grafana
    restart: unless-stopped
    ports:
      - "127.0.0.1:3001:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana:/etc/grafana/provisioning:ro
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=your-secure-password
    profiles:
      - monitoring

volumes:
  grafana-data:
EOF

# 启动节点
docker-compose up -d

# 查看日志
docker-compose logs -f yrust-node
```

### 步骤 7: 配置监控

创建 `monitoring/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'yrust-node'
    static_configs:
      - targets: ['yrust-node:3000']
    metrics_path: /metrics
```

### 步骤 8: 设置备份

```bash
# 创建备份脚本
cat > /opt/blockchain/backup.sh << 'EOF'
#!/bin/bash

BACKUP_DIR="/opt/blockchain/backups"
DATE=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=7

# 创建备份目录
mkdir -p $BACKUP_DIR

# 停止节点（可选，确保数据一致性）
# docker-compose stop yrust-node

# 备份数据
tar czf $BACKUP_DIR/yrust_backup_$DATE.tar.gz -C /opt/blockchain data config

# 重启节点
# docker-compose start yrust-node

# 上传到远程存储（可选）
# aws s3 cp $BACKUP_DIR/yrust_backup_$DATE.tar.gz s3://your-bucket/backups/

# 清理旧备份
find $BACKUP_DIR -name "yrust_backup_*.tar.gz" -mtime +$RETENTION_DAYS -delete

echo "备份完成: $BACKUP_DIR/yrust_backup_$DATE.tar.gz"
EOF

chmod +x /opt/blockchain/backup.sh

# 设置定时任务
crontab -e
# 添加：
# 0 2 * * * /opt/blockchain/backup.sh >> /var/log/yrust-backup.log 2>&1
```

### 步骤 9: 启动种子节点

```bash
# 配置种子节点
cat > config/seed.toml << 'EOF'
[network]
node_type = "seed"
listen_addr = "/ip4/0.0.0.0/tcp/8000"
bootstrap_nodes = []

[api]
enabled = true
port = 3000
EOF

# 启动
docker-compose -f docker-compose.yml -f docker-compose.seed.yml up -d
```

### 步骤 10: 验证部署

```bash
# 检查节点状态
curl https://yrust.chain/api/stats

# 检查区块同步
curl https://yrust.chain/api/blocks

# 检查 P2P 连接
docker-compose exec yrust-node netstat -an | grep 8000

# 查看日志
docker-compose logs --tail 100 yrust-node
```

## 🔐 安全加固

### 1. SSH 安全配置

```bash
# 编辑 /etc/ssh/sshd_config
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
MaxAuthTries 3
ClientAliveInterval 300
ClientAliveCountMax 2

# 重启 SSH
sudo systemctl restart sshd
```

### 2. 配置 Fail2Ban

```bash
sudo tee /etc/fail2ban/jail.local << 'EOF'
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 3

[sshd]
enabled = true
port = 22
filter = sshd
logpath = /var/log/auth.log
EOF

sudo systemctl restart fail2ban
```

### 3. 系统监控

```bash
# 安装监控工具
sudo apt install -y htop iotop nethogs

# 配置日志轮转
sudo tee /etc/logrotate.d/yrust-chain << 'EOF'
/opt/blockchain/logs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 root root
    postrotate
        docker-compose -f /opt/blockchain/yrust-chain/docker-compose.yml kill -s USR1 yrust-node
    endscript
}
EOF
```

## 📊 运维命令

```bash
# 查看节点状态
docker-compose ps

# 重启节点
docker-compose restart yrust-node

# 更新节点
docker-compose pull
docker-compose up -d

# 查看资源使用
docker stats yrust-mainnet

# 清理日志
docker-compose exec yrust-node sh -c "truncate -s 0 /app/logs/*.log"

# 数据备份
docker-compose exec yrust-node tar czf /tmp/backup.tar.gz /app/data
```

## 🚨 故障排查

### 节点无法启动

```bash
# 检查日志
docker-compose logs yrust-node

# 检查端口占用
sudo netstat -tlnp | grep 3000

# 检查磁盘空间
df -h

# 检查内存
free -h
```

### 同步缓慢

```bash
# 检查网络连接
docker-compose exec yrust-node netstat -an

# 检查对等节点数
curl http://localhost:3000/api/peers

# 重启同步
docker-compose restart yrust-node
```

## 🎉 完成！

你的 YRUST Chain 主网已经部署完成！

访问地址：
- 区块浏览器: https://yrust.chain
- API: https://yrust.chain/api
- 监控: https://yrust.chain:3001 (Grafana)

下一步：
1. 邀请社区成员运行节点
2. 部署 DEX 和 NFT 合约
3. 启动生态建设
