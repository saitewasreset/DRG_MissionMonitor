# frontend

# build stage
FROM node:22-alpine AS fbase
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY ./frontend /app
WORKDIR /app

FROM fbase AS fprod-deps
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --prod --frozen-lockfile

FROM fbase AS fbuild
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

# backend
# build stage
FROM rust:alpine AS builder
WORKDIR /usr/src/mission-backend-rs
COPY ./backend/migrations ./migrations
COPY ./backend/src ./src
COPY ./backend/Cargo.toml ./Cargo.toml
COPY ./backend/Cargo.lock ./Cargo.lock
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
COPY --from=fbuild /app/dist /static
CMD ["mission-backend-rs"]