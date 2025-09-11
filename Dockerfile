# Multi-stage build for Cognitord daemon
# Stage 1: Build
FROM rust:1.80-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src/main.rs for dependencies caching
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release

# Remove dummy src
RUN rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM debian:12-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    systemd \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -d /app cognitord

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/cognitord /usr/local/bin/

# Copy systemd service and socket files
COPY systemd/cognitord.service /etc/systemd/system/
COPY systemd/cognitord.socket /etc/systemd/system/

# Copy example configuration
COPY docker/example-config.json /app/config.json.example

# Create necessary directories
RUN mkdir -p /var/lib/cognitord /var/log/cognitord /etc/cognitord /run/cognitord && \
    chown cognitord:cognitord /var/lib/cognitord /var/log/cognitord /etc/cognitord /run/cognitord

# Set permissions
RUN chmod +x /usr/local/bin/cognitord

# Create entrypoint script
COPY docker/entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/entrypoint.sh

# Switch to non-root user
USER cognitord

# No ports exposed (stdin/stdout only)

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD pgrep cognitord || exit 1

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

# Default command
CMD ["daemon"]