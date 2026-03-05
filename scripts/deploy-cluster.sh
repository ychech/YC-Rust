#!/bin/bash
# YRUST Chain 生产集群部署脚本
# 支持多服务器分布式部署

set -e

# 颜色定义
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
PROJECT_NAME="yrust-chain"
INSTALL_DIR="/opt/${PROJECT_NAME}"
DOCKER_NETWORK="yrust-cluster"

# 检查 root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "请使用 root 权限运行"
        exit 1
    fi
}

# 安装 Docker
install_docker() {
    log_info "安装 Docker..."
    
    if ! command -v docker &> /dev/null; then
        curl -fsSL https://get.docker.com | bash
        systemctl enable docker
        systemctl start docker
        log_success "Docker 安装完成"
    else
        log_info "Docker 已安装"
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        curl -L "https://github.com/docker/compose/releases/download/v2.23.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        chmod +x /usr/local/bin/docker-compose
        log_success "Docker Compose 安装完成"
    fi
}

# 创建目录结构
setup_directories() {
    log_info "创建目录结构..."
    
    mkdir -p ${INSTALL_DIR}/{config,logs,data,ssl}
    mkdir -p ${INSTALL_DIR}/config/{nginx,prometheus,grafana,alertmanager,logstash}
    mkdir -p ${INSTALL_DIR}/logs/{seed,validator1,validator2,fullnode1}
    
    chown -R 1000:1000 ${INSTALL_DIR}/logs
    chmod 755 ${INSTALL_DIR}/logs
    
    log_success "目录创建完成"
}

# 生成配置文件
generate_configs() {
    log_info "生成配置文件..."
    
    # Nginx配置
    cat > ${INSTALL_DIR}/config/nginx/nginx.conf << 'EOF'
events {
    worker_connections 4096;
}

http {
    upstream api_backend {
        least_conn;
        server fullnode-1:8080 max_fails=3 fail_timeout=30s;
        server validator-1:8080 backup;
        server validator-2:8080 backup;
    }
    
    server {
        listen 80;
        server_name _;
        
        location / {
            root /usr/share/nginx/html;
            index index.html;
            try_files $uri $uri/ /index.html;
        }
        
        location /api/ {
            proxy_pass http://api_backend/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            
            # 速率限制
            limit_req zone=api_limit burst=100 nodelay;
            limit_req_status 429;
        }
        
        location /health {
            proxy_pass http://api_backend/health;
            access_log off;
        }
    }
}
EOF

    # Prometheus配置
    cat > ${INSTALL_DIR}/config/prometheus/prometheus.yml << EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

rule_files:
  - /etc/prometheus/rules/*.yml

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'yrust-nodes'
    static_configs:
      - targets: 
        - 'seed-node:8080'
        - 'validator-1:8080'
        - 'validator-2:8080'
        - 'fullnode-1:8080'
    metrics_path: /metrics
    scrape_interval: 5s
EOF

    # 告警规则
    mkdir -p ${INSTALL_DIR}/config/prometheus/rules
    cat > ${INSTALL_DIR}/config/prometheus/rules/yrust.yml << 'EOF'
groups:
  - name: yrust-alerts
    rules:
      - alert: NodeDown
        expr: up{job="yrust-nodes"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "YRUST节点 {{ \$labels.instance }} 下线"
          description: "节点已经下线超过1分钟"
      
      - alert: BlockHeightStuck
        expr: rate(block_height[5m]) == 0
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "区块高度停滞"
          description: "10分钟内没有新区块产生"
      
      - alert: HighMemoryUsage
        expr: memory_usage_bytes / memory_total_bytes > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "内存使用率过高"
          description: "内存使用率超过90%"
EOF

    # Alertmanager配置
    cat > ${INSTALL_DIR}/config/alertmanager/alertmanager.yml << 'EOF'
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alert@yrust.chain'

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'

receivers:
  - name: 'web.hook'
    webhook_configs:
      - url: 'http://localhost:5001/webhook'
        send_resolved: true
EOF

    log_success "配置文件生成完成"
}

# 配置防火墙
setup_firewall() {
    log_info "配置防火墙..."
    
    if command -v ufw &> /dev/null; then
        ufw default deny incoming
        ufw default allow outgoing
        ufw allow 22/tcp      # SSH
        ufw allow 80/tcp      # HTTP
        ufw allow 443/tcp     # HTTPS
        ufw allow 8080:8083/tcp  # API
        ufw allow 9090:9095/tcp  # P2P + Metrics
        ufw allow 3000/tcp    # Grafana
        ufw allow 5601/tcp    # Kibana
        ufw --force enable
        log_success "UFW配置完成"
    fi
}

# 启动集群
start_cluster() {
    log_info "启动 YRUST Chain 集群..."
    
    cd ${INSTALL_DIR}
    
    # 拉取最新镜像
    docker-compose -f docker-compose.prod.yml pull
    
    # 启动服务
    docker-compose -f docker-compose.prod.yml up -d
    
    # 等待服务就绪
    log_info "等待服务就绪..."
    sleep 30
    
    # 检查健康状态
    docker-compose -f docker-compose.prod.yml ps
    
    log_success "集群启动完成"
}

# 显示状态
show_status() {
    echo ""
    echo "========================================"
    echo "  YRUST Chain 集群部署完成!"
    echo "========================================"
    echo ""
    echo "服务地址:"
    echo "  官网:      http://$(curl -s ifconfig.me)"
    echo "  API:       http://$(curl -s ifconfig.me):8080"
    echo "  监控:      http://$(curl -s ifconfig.me):3000"
    echo "  日志:      http://$(curl -s ifconfig.me):5601"
    echo ""
    echo "管理命令:"
    echo "  查看日志:  docker-compose -f docker-compose.prod.yml logs -f"
    echo "  停止集群:  docker-compose -f docker-compose.prod.yml down"
    echo "  重启服务:  docker-compose -f docker-compose.prod.yml restart"
    echo ""
    echo "========================================"
}

# 主函数
main() {
    echo "========================================"
    echo "  YRUST Chain 集群部署脚本"
    echo "========================================"
    echo ""
    
    check_root
    install_docker
    setup_directories
    generate_configs
    setup_firewall
    start_cluster
    show_status
}

main "$@"
