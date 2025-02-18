# 第一阶段：构建 Rust 项目
FROM rust:1.80.1-alpine3.20 AS rust-builder
WORKDIR /telegram-onedrive
COPY ./ ./
RUN apk add --update --no-cache build-base pkgconfig libressl-dev && \
    cargo build --release

# 第二阶段：安装证书
FROM alpine:3.20 AS certs
RUN apk add --no-cache ca-certificates

# 第三阶段：构建最终镜像
FROM alpine:3.20
# 复制构建产物
COPY --from=rust-builder /telegram-onedrive/target/release/telegram-onedrive /
COPY --from=rust-builder /telegram-onedrive/index.html /
# 复制证书
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
# 安装运行时依赖
RUN apk add --update --no-cache bash curl net-tools
ENV RUST_BACKTRACE=1

ENTRYPOINT [ "/telegram-onedrive" ]
