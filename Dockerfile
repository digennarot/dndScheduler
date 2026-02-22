# Build Stage
FROM docker.io/library/rust:latest as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock sqlx-data.jso[n] ./

# Build caching trick (optional but recommended, simplified here for robustness)
# Simple copy source and build
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime Stage
FROM docker.io/library/debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/dnd_scheduler /app/dnd_scheduler

# Copy static assets
COPY static /app/static

# Copy example env (optional, user should mount real .env)
COPY .env.example /app/.env.example

# Set environment
ENV RUST_LOG=info
ENV STATIC_DIR=/app/static
ENV PORT=3000
ENV HOST=0.0.0.0

# Expose port
EXPOSE 3000

# Run
CMD ["./dnd_scheduler"]
