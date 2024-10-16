# build stage
FROM rust:alpine AS builder
WORKDIR /usr/src/mission-backend-rs
COPY ./migrations ./migrations
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN apk add --no-cache musl-dev pkgconf openssl-dev libpq-dev openssl-libs-static
ENV OPENSSL_STATIC=true \
    OPENSSL_LIB_DIR=/usr/lib \
    OPENSSL_INCLUDE_DIR=/usr/include \
    PKG_CONFIG_ALLOW_CROSS=1 \
    PQ_LIB_STATIC=true
RUN cargo build --release --bin mission-backend-rs
# https://www.aloxaf.com/2018/09/reduce_rust_size/
RUN apk add --no-cache binutils upx
RUN strip target/release/mission-backend-rs
RUN upx --best target/release/mission-backend-rs

# production stage
FROM alpine:latest
RUN apk add --no-cache curl
COPY --from=builder /usr/src/mission-backend-rs/target/release/mission-backend-rs /usr/local/bin/mission-backend-rs
CMD ["mission-backend-rs"]
