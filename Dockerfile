# ==============================================================================
# Multi-Stage Dockerfile for Vercel Container Deployment (Fluid Compute)
# ==============================================================================

# ------------------------------------------------------------------------------
# Stage 1: Build binary using official Rust bookworm
# ------------------------------------------------------------------------------
FROM rust:bookworm AS builder

WORKDIR /usr/src/sealed-books

# 1. Install build tools, OpenSSL, Clang (for libsql bindgen), and curl/unzip for protoc
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    curl \
    unzip \
    ca-certificates \
    clang \
    && rm -rf /var/lib/apt/lists/*

# 2. Install modern protoc (Debian bookworm apt packages protoc 21.12 which breaks hedera-proto include paths)
RUN curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v29.3/protoc-29.3-linux-x86_64.zip \
    && unzip protoc-29.3-linux-x86_64.zip -d /usr/local \
    && rm protoc-29.3-linux-x86_64.zip \
    && protoc --version

# 3. Generate a clean workspace Cargo.toml with ONLY core and server (no desktop or harness)
RUN printf '[workspace]\nresolver = "3"\nmembers = [\n    "crates/core",\n    "apps/server",\n]\n\n[workspace.package]\nversion = "0.1.0"\nedition = "2024"\nlicense = "MIT"\nauthors = ["Vimal Saraswat"]\n\n[workspace.dependencies]\nsealed-books-core = { path = "crates/core" }\nserde = { version = "1", features = ["derive"] }\nserde_json = "1"\n\n[profile.release]\ncodegen-units = 1\nlto = true\nopt-level = 3\npanic = "abort"\nstrip = true\n' > Cargo.toml

# 4. Copy Cargo.lock and the required server crates
COPY Cargo.lock ./
COPY crates/core/ crates/core/
COPY apps/server/ apps/server/

# 5. Build optimized release binary
RUN cargo build --release --bin sealed-books-server

# ------------------------------------------------------------------------------
# Stage 2: Minimal runtime image for Vercel Fluid Compute / Cloud Deployment
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Install runtime SSL certificates (required for Turso HTTPS, Hedera Mirror Node & SMTP TLS)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from builder
COPY --from=builder /usr/src/sealed-books/target/release/sealed-books-server /app/server

# Standard container port fallback (platforms like Vercel inject PORT at runtime)
EXPOSE 8080
ENV PORT=8080

# Run the Axum server (reads DATABASE_URL, TURSO_AUTH_TOKEN, etc. from runtime environment)
CMD ["/app/server"]
