# 第一阶段：构建 Rust 项目
FROM rust:1.80.1-alpine3.20 AS rust-builder
WORKDIR /telegram-onedrive
COPY ./ ./
RUN apk add --update --no-cache build-base pkgconfig libressl-dev &&\
    cargo build --release

# 第二阶段：安装证书
FROM alpine:3.20 as certs

# 第三阶段：构建最终镜像
FROM scratch
COPY --from=rust-builder /telegram-onedrive/target/release/telegram-onedrive /
COPY --from=rust-builder /telegram-onedrive/index.html /
COPY --from=certs /etc/ssl/cert.pem /etc/ssl/
ENV RUST_BACKTRACE=1

# 在最终镜像中安装所需的工具
RUN apk add --update --no-cache bash curl net-tools

ENTRYPOINT [ "/telegram-onedrive" ]
