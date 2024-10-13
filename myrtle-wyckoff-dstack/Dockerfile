# Use the official Rust 1.81 image for building the binary
FROM rust:1.81 as builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Install necessary libraries and tools
RUN apt-get update && \
    apt-get install -y build-essential pkg-config libssl-dev

# Copy over Cargo configuration files to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src

# Pre-fetch dependencies to leverage Docker caching
RUN cargo fetch

# Copy the rest of the project and build it in release mode
COPY . .
RUN cargo build --release

# Use Ubuntu 22.04 as the base image for the final container
FROM ubuntu:22.04

# Install necessary runtime libraries
RUN apt-get update && \
    apt-get install -y libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Set environment variables for Rocket in production mode
ENV ROCKET_ENV=production

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/myrtle-wyckoff-dstack /usr/local/bin/app

# Expose the port Rocket will use
EXPOSE 8000

# Run the Rocket server
CMD ["app"]
