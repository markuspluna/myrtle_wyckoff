# Build stage
FROM rust:1.75-slim as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ ./src/

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates cryptsetup && \
    rm -rf /var/lib/apt/lists/*

# Create directory for encrypted storage
RUN mkdir -p /app/encrypted_data

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/myrtle-wyckoff-dstack .

# Expose the port the app runs on
EXPOSE 8000

VOLUME ["/app/encrypted_data"]

# Command to run the application
CMD ["./myrtle-wyckoff-dstack"]
