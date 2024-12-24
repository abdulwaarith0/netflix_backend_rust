# Build stage
FROM rust:1.80-bullseye AS builder

WORKDIR /netflix_backend_rust

# Install build dependencies
RUN apt-get update && apt-get install -y build-essential libssl-dev pkg-config

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Copy the rest of the source code
COPY . .

# Build the actual application
RUN cargo build --release

# Final stage
FROM debian:bullseye-slim

WORKDIR /netflix_backend_rust

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /netflix_backend_rust/target/release/netflix_backend_rust /usr/local/bin/

# Expose the application port
EXPOSE 8080

# Command to run the application
CMD ["netflix_backend_rust"]