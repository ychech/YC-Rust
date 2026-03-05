#!/bin/bash

# YRust Chain 部署脚本
# 支持：Docker、AWS、阿里云、腾讯云

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 配置
IMAGE_NAME="yrust-chain"
VERSION="latest"
CONTAINER_NAME="yrust-node"

echo -e "${GREEN}🚀 YRust Chain 部署脚本${NC}"
echo ""

# 检查命令
function check_command() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}错误: $1 未安装${NC}"
        exit 1
    fi
}

# 本地 Docker 部署
function deploy_local() {
    echo -e "${YELLOW}📦 本地 Docker 部署...${NC}"
    
    check_command docker
    check_command docker-compose
    
    # 构建镜像
    echo "🔨 构建 Docker 镜像..."
    docker build -t $IMAGE_NAME:$VERSION -f Dockerfile ..
    
    # 启动服务
    echo "🚀 启动服务..."
    docker-compose up -d
    
    # 等待服务启动
    echo "⏳ 等待服务启动..."
    sleep 5
    
    # 检查健康状态
    if curl -f http://localhost:3000/api/stats > /dev/null 2>&1; then
        echo -e "${GREEN}✅ 部署成功！${NC}"
        echo ""
        echo "访问地址:"
        echo "  区块浏览器: http://localhost"
        echo "  API: http://localhost:3000"
        echo ""
        echo "查看日志: docker-compose logs -f"
    else
        echo -e "${RED}❌ 部署失败，请检查日志${NC}"
        docker-compose logs
    fi
}

# 部署到云服务器
function deploy_cloud() {
    local host=$1
    local user=$2
    local key=$3
    
    echo -e "${YELLOW}☁️  部署到云服务器: $host${NC}"
    
    # 检查参数
    if [ -z "$host" ] || [ -z "$user" ]; then
        echo -e "${RED}错误: 请提供服务器地址和用户名${NC}"
        echo "用法: ./deploy.sh cloud <host> <user> [key_file]"
        exit 1
    fi
    
    # 打包项目
    echo "📦 打包项目..."
    cd ..
    tar -czf deploy/yrust-chain.tar.gz --exclude='target' --exclude='.git' --exclude='data' .
    cd deploy
    
    # 上传文件
    echo "📤 上传文件到服务器..."
    if [ -n "$key" ]; then
        scp -i $key yrust-chain.tar.gz $user@$host:/tmp/
        scp -i $key Dockerfile docker-compose.yml nginx.conf $user@$host:/tmp/
        ssh -i $key $user@$host "mkdir -p ~/yrust-chain && cd ~/yrust-chain && tar -xzf /tmp/yrust-chain.tar.gz && mv /tmp/Dockerfile /tmp/docker-compose.yml /tmp/nginx.conf ."
    else
        scp yrust-chain.tar.gz $user@$host:/tmp/
        scp Dockerfile docker-compose.yml nginx.conf $user@$host:/tmp/
        ssh $user@$host "mkdir -p ~/yrust-chain && cd ~/yrust-chain && tar -xzf /tmp/yrust-chain.tar.gz && mv /tmp/Dockerfile /tmp/docker-compose.yml /tmp/nginx.conf ."
    fi
    
    # 远程执行部署
    echo "🔨 在服务器上构建和启动..."
    if [ -n "$key" ]; then
        ssh -i $key $user@$host "cd ~/yrust-chain && docker-compose down && docker-compose up --build -d"
    else
        ssh $user@$host "cd ~/yrust-chain && docker-compose down && docker-compose up --build -d"
    fi
    
    echo -e "${GREEN}✅ 云服务器部署完成！${NC}"
    echo "访问地址: http://$host"
}

# 停止服务
function stop() {
    echo -e "${YELLOW}🛑 停止服务...${NC}"
    docker-compose down
    echo -e "${GREEN}✅ 服务已停止${NC}"
}

# 查看日志
function logs() {
    docker-compose logs -f
}

# 更新代码
function update() {
    echo -e "${YELLOW}🔄 更新代码...${NC}"
    
    # 拉取最新代码
    git pull
    
    # 重新构建和启动
    docker-compose down
    docker-compose up --build -d
    
    echo -e "${GREEN}✅ 更新完成${NC}"
}

# 备份数据
function backup() {
    local backup_dir="backup_$(date +%Y%m%d_%H%M%S)"
    echo -e "${YELLOW}💾 备份数据到 $backup_dir...${NC}"
    
    mkdir -p $backup_dir
    docker run --rm -v yrust-data:/data -v $(pwd)/$backup_dir:/backup alpine tar czf /backup/data.tar.gz -C /data .
    
    echo -e "${GREEN}✅ 备份完成: $backup_dir/data.tar.gz${NC}"
}

# 恢复数据
function restore() {
    local backup_file=$1
    
    if [ -z "$backup_file" ]; then
        echo -e "${RED}错误: 请指定备份文件${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}📥 恢复数据从 $backup_file...${NC}"
    
    # 停止服务
    docker-compose down
    
    # 恢复数据
    docker run --rm -v yrust-data:/data -v $(pwd)/$backup_file:/backup.tar.gz alpine tar xzf /backup.tar.gz -C /data
    
    # 启动服务
    docker-compose up -d
    
    echo -e "${GREEN}✅ 恢复完成${NC}"
}

# 显示帮助
function show_help() {
    echo "YRust Chain 部署脚本"
    echo ""
    echo "用法: ./deploy.sh [命令] [参数]"
    echo ""
    echo "命令:"
    echo "  local                    本地 Docker 部署"
    echo "  cloud <host> <user> [key] 部署到云服务器"
    echo "  stop                     停止服务"
    echo "  logs                     查看日志"
    echo "  update                   更新代码并重启"
    echo "  backup                   备份数据"
    echo "  restore <file>           恢复数据"
    echo ""
    echo "示例:"
    echo "  ./deploy.sh local"
    echo "  ./deploy.sh cloud 1.2.3.4 root ~/.ssh/id_rsa"
    echo "  ./deploy.sh backup"
}

# 主逻辑
case "${1:-local}" in
    local)
        deploy_local
        ;;
    cloud)
        deploy_cloud $2 $3 $4
        ;;
    stop)
        stop
        ;;
    logs)
        logs
        ;;
    update)
        update
        ;;
    backup)
        backup
        ;;
    restore)
        restore $2
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo -e "${RED}未知命令: $1${NC}"
        show_help
        exit 1
        ;;
esac
