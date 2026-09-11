# syntax=docker/dockerfile:1

# --- Build stage ---
# Rust 1.85 is the first release with Rust 2024 edition support. Keep a
# default here because Fly builds do not supply this build argument.
ARG RUST_VERSION=1.85
FROM rust:${RUST_VERSION}-bookworm AS builder

WORKDIR /app

ENV PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1

# Fly's dashboard builds from the repository root. Copy from the `crystal/`
# workspace while retaining that workspace's layout inside the image.
# Copy manifests first so Docker can reuse the dependency layer.
COPY crystal/Cargo.toml crystal/Cargo.lock ./
COPY crystal/crystal-core/Cargo.toml ./crystal-core/Cargo.toml
COPY crystal/crystal-server/Cargo.toml ./crystal-server/Cargo.toml
COPY crystal/crystal-cli/Cargo.toml ./crystal-cli/Cargo.toml
RUN mkdir -p crystal-core/src crystal-server/src crystal-cli/src \
    && printf 'pub fn placeholder() {}\n' > crystal-core/src/lib.rs \
    && printf 'fn main() {}\n' > crystal-server/src/main.rs \
    && printf 'pub fn placeholder() {}\n' > crystal-server/src/lib.rs \
    && printf 'fn main() {}\n' > crystal-cli/src/main.rs
RUN cargo fetch --locked

# Build the CLI release binary from the workspace. The server runs as `scryr serve`.
COPY crystal/crystal-core ./crystal-core
COPY crystal/crystal-server ./crystal-server
COPY crystal/crystal-cli ./crystal-cli
RUN cargo build --locked --release -p crystal-cli --bin scryr

# --- Runtime stage ---
FROM ubuntu:26.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/scryr /usr/local/bin/scryr

# Bind to all interfaces inside the container so Docker port-mapping works.
# Override HOST=127.0.0.1 at runtime for local non-containerised dev.
ENV HOST=0.0.0.0
ENV PORT=8000

EXPOSE 8000

ENTRYPOINT ["scryr", "serve", "--server-only"]
