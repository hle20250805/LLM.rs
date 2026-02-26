# 第一阶段：构建阶段
FROM rust:1.88 as builder

# 设置工作目录
WORKDIR /app

# 复制 Cargo.toml 和 Cargo.lock 文件
COPY Cargo.toml Cargo.lock ./

# 复制 src 目录
COPY src ./src

# 构建项目（使用 --release 模式以获得最佳性能）
RUN cargo build --release

# 第二阶段：运行阶段
FROM debian:bookworm-slim

# 安装必要的依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 从构建阶段复制编译好的可执行文件
COPY --from=builder /app/target/release/llmrs ./

# 复制配置文件
COPY config.toml ./

# 暴露端口（根据实际使用的端口调整）
EXPOSE 3000

# 设置环境变量
ENV RUST_LOG=info

# 运行应用
CMD ["./llmrs"]