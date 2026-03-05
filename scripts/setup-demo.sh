#!/bin/bash
# YRUST Chain 公网 Demo 部署脚本
# 一键部署到公网可访问的演示环境

set -e

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# 配置
DOMAIN="${1:-demo.yrust.chain}"
EMAIL="${2:-admin@yrust.chain}"
INSTALL_DIR="/opt/yrust-demo"

log_info "开始部署 YRUST Chain 公网 Demo"
log_info "域名: $DOMAIN"
log_info "邮箱: $EMAIL"

# 检查 root
if [[ $EUID -ne 0 ]]; then
    log_error "请使用 root 权限运行"
    exit 1
fi

# 安装依赖
log_info "安装依赖..."
apt-get update
apt-get install -y docker.io docker-compose nginx certbot python3-certbot-nginx ufw curl

# 启动 Docker
systemctl enable docker
systemctl start docker

# 创建目录
mkdir -p $INSTALL_DIR
cd $INSTALL_DIR

# 克隆代码（如果还没有）
if [[ ! -d "y-rust" ]]; then
    log_info "克隆代码..."
    git clone https://github.com/yourusername/y-rust.git || true
fi

cd y-rust

# 构建镜像
log_info "构建 Docker 镜像..."
docker-compose build

# 配置防火墙
log_info "配置防火墙..."
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 8080/tcp
ufw allow 9090/tcp
ufw --force enable

# 配置 Nginx
log_info "配置 Nginx..."
cat > /etc/nginx/sites-available/yrust-demo << EOF
server {
    listen 80;
    server_name $DOMAIN;
    
    location / {
        proxy_pass http://localhost:80;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
    }
    
    location /api/ {
        proxy_pass http://localhost:8080/;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_read_timeout 300s;
    }
    
    location /ws {
        proxy_pass http://localhost:8080/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
EOF

ln -sf /etc/nginx/sites-available/yrust-demo /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default
nginx -t
systemctl restart nginx

# 申请 SSL 证书
log_info "申请 SSL 证书..."
certbot --nginx -d $DOMAIN --non-interactive --agree-tos --email $EMAIL || true

# 创建 systemd 服务
log_info "创建系统服务..."
cat > /etc/systemd/system/yrust-demo.service << EOF
[Unit]
Description=YRUST Chain Demo
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=$INSTALL_DIR/y-rust
ExecStart=/usr/bin/docker-compose up -d
ExecStop=/usr/bin/docker-compose down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable yrust-demo

# 启动服务
log_info "启动服务..."
docker-compose up -d

# 等待服务就绪
log_info "等待服务就绪..."
sleep 30

# 显示状态
echo ""
echo "========================================"
echo "  YRUST Chain 公网 Demo 部署完成!"
echo "========================================"
echo ""
echo "访问地址:"
echo "  官网:    https://$DOMAIN"
echo "  API:     https://$DOMAIN/api"
echo "  浏览器:  https://$DOMAIN/explorer"
echo "  钱包:    https://$DOMAIN/wallet"
echo "  DEX:     https://$DOMAIN/dex"
echo ""
echo "管理命令:"
echo "  查看状态: docker-compose ps"
echo "  查看日志: docker-compose logs -f"
echo "  重启服务: docker-compose restart"
echo "  停止服务: docker-compose down"
echo ""
echo "========================================"
