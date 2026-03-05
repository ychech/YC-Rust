# YRust Chain 生产环境 Dockerfile

# 构建阶段
FROM rust:1.75-slim as builder

WORKDIR /app

# 安装依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    clang \
    llvm \
    && rm -rf /var/lib/apt/lists/*

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟 main.rs 以缓存依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# 复制源代码
COPY src ./src
COPY web ./web

# 构建（使用缓存的依赖）
RUN touch src/main.rs
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 复制构建产物
COPY --from=builder /app/target/release/yrust_chain /app/yrust_chain
COPY --from=builder /app/web /app/web

# 创建数据目录
RUN mkdir -p /app/data

# 暴露端口
EXPOSE 3000 8000

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/stats || exit 1

# 启动命令
ENTRYPOINT ["/app/yrust_chain"]
CMD ["node"]
