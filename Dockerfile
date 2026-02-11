# =============================================================================
# AI Model Vault — Multi-stage Dockerfile
# Produces a minimal, statically-linked binary for production use.
#
# Build targets:
#   docker build -t aim:latest .                         # default (alpine)
#   docker build --build-arg FEATURES=api -t aim:api .   # with REST API
#   docker build --target debian -t aim:debian .         # Debian slim variant
# =============================================================================

# ---------------------------------------------------------------------------
# Stage 1 — Builder (uses Rust slim to compile the binary)
# ---------------------------------------------------------------------------
FROM rust:1.82-slim-bookworm AS builder

ARG FEATURES=""

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev musl-tools && \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY src/ src/
COPY benches/ benches/
COPY examples/ examples/
COPY tests/ tests/

# Build the release binary (statically linked via musl)
RUN if [ -z "$FEATURES" ]; then \
      cargo build --release --target x86_64-unknown-linux-musl; \
    else \
      cargo build --release --target x86_64-unknown-linux-musl --features "$FEATURES"; \
    fi

# ---------------------------------------------------------------------------
# Stage 2a — Alpine runtime (default, ~12 MB image)
# ---------------------------------------------------------------------------
FROM alpine:3.20 AS alpine

RUN apk add --no-cache ca-certificates tini && \
    addgroup -g 1000 aim && \
    adduser -u 1000 -G aim -s /sbin/nologin -D aim

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/aim /usr/local/bin/aim
RUN chmod +x /usr/local/bin/aim

# XDG base directories
ENV XDG_DATA_HOME=/data \
    XDG_CONFIG_HOME=/config \
    XDG_CACHE_HOME=/cache

RUN mkdir -p /data /config /cache && chown -R aim:aim /data /config /cache

USER aim
WORKDIR /data

ENTRYPOINT ["tini", "--"]
CMD ["aim"]

EXPOSE 8080
VOLUME ["/data", "/config", "/cache"]

LABEL org.opencontainers.image.title="AI Model Vault" \
      org.opencontainers.image.description="Universal secure vault for AI model formats" \
      org.opencontainers.image.version="1.0.0" \
      org.opencontainers.image.source="https://github.com/nervosys/ai-model-vault" \
      org.opencontainers.image.licenses="AGPL-3.0-or-later"

# ---------------------------------------------------------------------------
# Stage 2b — Debian runtime (for environments requiring glibc)
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS debian

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates tini && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -g 1000 aim && \
    useradd -u 1000 -g aim -s /usr/sbin/nologin -m aim

# For the Debian variant we rebuild with the default glibc target
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/aim /usr/local/bin/aim
RUN chmod +x /usr/local/bin/aim

ENV XDG_DATA_HOME=/data \
    XDG_CONFIG_HOME=/config \
    XDG_CACHE_HOME=/cache

RUN mkdir -p /data /config /cache && chown -R aim:aim /data /config /cache

USER aim
WORKDIR /data

ENTRYPOINT ["tini", "--"]
CMD ["aim"]

EXPOSE 8080
VOLUME ["/data", "/config", "/cache"]

LABEL org.opencontainers.image.title="AI Model Vault" \
      org.opencontainers.image.description="Universal secure vault for AI model formats" \
      org.opencontainers.image.version="1.0.0" \
      org.opencontainers.image.source="https://github.com/nervosys/ai-model-vault" \
      org.opencontainers.image.licenses="AGPL-3.0-or-later"
