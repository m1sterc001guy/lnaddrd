# ---- Build Stage ----
FROM rust:1.86 AS builder
WORKDIR /app

# Install build dependencies for diesel/postgres
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# Build actual source
COPY . .
RUN cargo build --release

# ---- Runtime Stage ----
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies for diesel/postgres
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder
COPY --from=builder /app/target/release/lnaddrd /usr/local/bin/lnaddrd

# Expose the default port
EXPOSE 8080

# Set environment variables for configuration (can be overridden)
ENV LNADDRD_BIND=0.0.0.0:8080

# Entrypoint
ENTRYPOINT ["/usr/local/bin/lnaddrd"]