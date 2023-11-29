# Use a Rust base image
FROM rust:latest as builder

# Install CMake, libc
RUN apt update && \
    apt install -y cmake && \
    rm -rf /var/lib/apt/lists/*

#WORKDIR /app

# Copy the cargo manifest and lock file for dependency resolution
#COPY Cargo.toml Cargo.lock ./

# Build dependencies separately to leverage Docker caching
#RUN cargo build --release && \
#    rm -f target/release/deps/dummy*

WORKDIR /app

# Copy the source code
COPY . .

# Build the Rust project
RUN cargo build --release

#COPY /app/target/release/inx-eddn .

# Create a new, smaller image without the build dependencies
FROM ubuntu-latest

WORKDIR /app

# Copy just the compiled binary from the previous stage
COPY --from=builder /app/target/release/inx-eddn /app/

# Set the entry point
CMD ["./inx-eddn"]