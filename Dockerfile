# ==============================================================================
# Multi-Stage Dockerfile for Sealed Books Axum Server
# ==============================================================================

# ------------------------------------------------------------------------------
# Stage 1: Build binary using official Rust bookworm
# ------------------------------------------------------------------------------
FROM rust:bookworm AS builder

WORKDIR /usr/src/sealed-books

# 1. Install build tools needed for protobuf & native crypto (used by hedera/tonic)
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 2. Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml crates/core/
COPY apps/server/Cargo.toml apps/server/
COPY apps/desktop/src-tauri/Cargo.toml apps/desktop/src-tauri/
COPY tools/harness/Cargo.toml tools/harness/

# 3. Copy Rust sources needed for server build
COPY crates/core/src crates/core/src
COPY apps/server/src apps/server/src

# 4. Build optimized release binary
RUN cargo build --release --bin sealed-books-server

# ------------------------------------------------------------------------------
# Stage 2: Minimal runtime image
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Install runtime SSL certificates (required for Hedera Mirror Node & SMTP TLS)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from builder
COPY --from=builder /usr/src/sealed-books/target/release/sealed-books-server /app/server

# Expose default HTTP port
EXPOSE 8080

ENV PORT=8080
ENV DATABASE_PATH=/app/data/sealed_books.db

# Create data directory for SQLite
RUN mkdir -p /app/data

# Run the Axum server
CMD ["/app/server"]
