# Dockerfile for testing Rust WASM compilation
# This Dockerfile sets up an environment to compile Rust code to WASM (WebAssembly).
# It is primarily used to test Rust WASM compilation in a clean environment.

# Use the official Rust image
FROM rust:latest

# Install dependencies necessary for compiling WASM
RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    llvm-dev \
    curl \
    bash \
    make

# Install wasm target for Rust
RUN rustup target add wasm32-unknown-unknown

# Create and set working directory inside the container
WORKDIR /workspace

# Copy the entire project into the container
COPY . .

# Build the Rust project for the wasm32 target
RUN cargo build --release --target wasm32-unknown-unknown

# Expose the target directory (where the WASM files are located)
# VOLUME ["/workspace/target/wasm32-unknown-unknown/release"]

# Set default command to keep the container running
# CMD ["tail", "-f", "/dev/null"]
