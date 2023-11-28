# Use a Rust base image
FROM rust:latest as builder

# Install CMake, libc
RUN apt update && \
    apt install -y cmake libc6 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy the cargo manifest and lock file for dependency resolution
COPY Cargo.toml Cargo.lock ./

# Build dependencies separately to leverage Docker caching
RUN mkdir src && \
    echo "fn main() {println!(\"dummy\")}" > src/main.rs && \
    cargo build --release && \
    rm -f target/release/deps/dummy*

# Copy the source code
COPY . .

# Build the Rust project
RUN cargo build --release

# Create a new, smaller image without the build dependencies
FROM debian:buster-slim

WORKDIR /usr/src/app

# Copy just the compiled binary from the previous stage
COPY --from=builder /usr/src/app/target/release/inx-eddn .

# Set the entry point
CMD ["./inx-eddn"]