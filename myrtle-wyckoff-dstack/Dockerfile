# 1. Use the official Rust image for building
FROM rust:1.72 as builder

# 2. Set the working directory inside the container
WORKDIR /usr/src/app

# 3. Copy the Cargo files and download dependencies (layer caching)
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# 4. Copy the entire project and build it in release mode
COPY . .
RUN cargo build --release

# 5. Use a lightweight image for the final container
FROM debian:buster-slim

# 6. Set environment variables for Rocket in production mode
ENV ROCKET_ENV=production

# 7. Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/rocket-docker /usr/local/bin/app

# 8. Expose the port Rocket will use
EXPOSE 8000

# 9. Run the Rocket application
CMD ["app"]
