# syntax=docker/dockerfile:1

FROM rust:1.96-bookworm AS builder

ARG WASM_PACK_VERSION=0.15.0

RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-pack --version "${WASM_PACK_VERSION}" --locked

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo fetch --target wasm32-unknown-unknown

COPY web ./web

RUN wasm-pack build --target web --out-dir web/pkg --release

FROM nginx:1.29-alpine AS runtime

COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/web/ /usr/share/nginx/html/

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -q --spider http://127.0.0.1/ || exit 1
