# ═══════════════════════════════════════════════════════════════
# MindFry - Dockerfile
# Multi-stage build for minimal production image
# ═══════════════════════════════════════════════════════════════

# Stage 1: Builder
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY benches/ ./benches/

# Build release binary
RUN cargo build --release --bin mindfry-server

# ═══════════════════════════════════════════════════════════════
# Stage 2: Runtime (minimal)
# ═══════════════════════════════════════════════════════════════
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 mindfry

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/mindfry-server /usr/local/bin/mindfry-server

# Create data directory
RUN mkdir -p /app/data && chown -R mindfry:mindfry /app

USER mindfry

# Default port
EXPOSE 9527

# Data volume
VOLUME ["/app/data"]

# Environment
ENV MINDFRY_DATA_DIR=/app/data
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD echo "" | nc -z localhost 9527 || exit 1

# Run server
ENTRYPOINT ["mindfry-server"]
CMD ["--host", "0.0.0.0", "--port", "9527", "--data-dir", "/app/data"]
