# ==============================================================================
# Multi-Stage Dockerfile for Sealed Books Axum Server (Standalone)
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

# 2. Generate a clean workspace Cargo.toml with ONLY core and server (no desktop or harness)
RUN printf '[workspace]\nresolver = "3"\nmembers = [\n    "crates/core",\n    "apps/server",\n]\n\n[workspace.package]\nversion = "0.1.0"\nedition = "2024"\nlicense = "MIT"\nauthors = ["Vimal Saraswat"]\n\n[workspace.dependencies]\nsealed-books-core = { path = "crates/core" }\nserde = { version = "1", features = ["derive"] }\nserde_json = "1"\n\n[profile.release]\ncodegen-units = 1\nlto = true\nopt-level = 3\npanic = "abort"\nstrip = true\n' > Cargo.toml

# 3. Copy Cargo.lock and the required server crates only
COPY Cargo.lock ./
COPY crates/core/ crates/core/
COPY apps/server/ apps/server/

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
