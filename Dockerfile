# YRUST Chain 节点 Docker 镜像
# 多阶段构建，优化镜像大小

# 阶段1: 构建
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    clang \
    llvm \
    && rm -rf /var/lib/apt/lists/*

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟main.rs以缓存依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# 复制源代码
COPY src ./src
COPY contracts ./contracts
COPY wallet ./wallet

# 构建应用
RUN touch src/main.rs && cargo build --release

# 阶段2: 运行时
FROM debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建非root用户
RUN groupadd -r yrust && useradd -r -g yrust yrust

# 复制构建产物
COPY --from=builder /app/target/release/y-rust /app/
COPY --from=builder /app/target/release/*.wasm /app/contracts/ 2>/dev/null || true

# 创建数据目录
RUN mkdir -p /data && chown yrust:yrust /data

# 复制配置文件
COPY config/docker.toml /app/config/node.toml 2>/dev/null || echo "# Default config" > /app/config/node.toml

# 设置权限
RUN chown -R yrust:yrust /app

# 切换到非root用户
USER yrust

# 暴露端口
# 8080: HTTP API
# 9090: P2P 网络
# 9091: 管理接口
EXPOSE 8080 9090 9091

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 数据卷
VOLUME ["/data"]

# 启动命令
ENTRYPOINT ["./y-rust"]
CMD ["--config", "/app/config/node.toml", "--data-dir", "/data"]
