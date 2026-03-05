#!/bin/bash
# YRUST Chain 一键部署脚本

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "请使用 root 权限运行此脚本"
        exit 1
    fi
}

install_deps() {
    log_info "安装依赖..."
    apt-get update
    apt-get install -y curl wget git docker.io docker-compose nginx certbot ufw
    systemctl enable docker
    systemctl start docker
    log_success "依赖安装完成"
}

setup_firewall() {
    log_info "配置防火墙..."
    ufw default deny incoming
    ufw default allow outgoing
    ufw allow 22/tcp 80/tcp 443/tcp 8080/tcp 9090/tcp
    ufw allow 3000:3004/tcp
    ufw --force enable
    log_success "防火墙配置完成"
}

deploy() {
    log_info "开始部署..."
    cd /opt/yrust-chain
    docker-compose up -d
    log_success "部署完成"
}

show_status() {
    echo "========================================"
    echo "YRUST Chain 部署完成!"
    echo "========================================"
    echo "访问地址:"
    echo "  浏览器: http://\$(curl -s ifconfig.me):3000"
    echo "  钱包:   http://\$(curl -s ifconfig.me):3001"
    echo "  DEX:    http://\$(curl -s ifconfig.me):3002"
    echo "========================================"
}

main() {
    check_root
    install_deps
    setup_firewall
    deploy
    show_status
}

main
