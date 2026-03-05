# 10. API与部署 - REST API和Docker

## 本章目标

实现REST API，学习Docker部署。

## REST API设计

### 核心端点
```
GET  /api/blocks              获取区块列表
GET  /api/blocks/:height      获取指定区块
GET  /api/tx/:id              获取交易
POST /api/tx                  发送交易
GET  /api/address/:addr       获取地址信息
GET  /api/stats               获取统计信息
```

## Docker部署

### Dockerfile
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/y-rust /usr/local/bin/
EXPOSE 8080
CMD ["y-rust"]
```

### Docker Compose
```yaml
version: '3.8'
services:
  node:
    build: .
    ports:
      - "8080:8080"
```

## 核心代码

见 [main.rs](main.rs)
