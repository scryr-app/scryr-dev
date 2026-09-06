# syntax=docker/dockerfile:1

# --- Build stage ---
FROM rust:1.98-bookworm AS builder

WORKDIR /app

ENV PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1

# Cache dependencies: copy workspace manifests first, then fetch deps
# before the full source tree so Docker can reuse the dependency layer.
COPY Cargo.toml Cargo.lock ./
COPY crystal-core/Cargo.toml ./crystal-core/Cargo.toml
COPY crystal-server/Cargo.toml ./crystal-server/Cargo.toml
COPY crystal-cli/Cargo.toml ./crystal-cli/Cargo.toml
RUN mkdir -p crystal-core/src crystal-server/src crystal-cli/src \
    && printf 'pub fn placeholder() {}\n' > crystal-core/src/lib.rs \
    && printf 'fn main() {}\n' > crystal-server/src/main.rs \
    && printf 'pub fn placeholder() {}\n' > crystal-server/src/lib.rs \
    && printf 'fn main() {}\n' > crystal-cli/src/main.rs
RUN cargo fetch

# Build the CLI release binary from the workspace. The server runs as `scryr serve`.
COPY crystal-core ./crystal-core
COPY crystal-server ./crystal-server
COPY crystal-cli ./crystal-cli
RUN cargo build --release -p crystal-cli --bin scryr

# --- Runtime stage ---
FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/scryr /usr/local/bin/scryr

# Bind to all interfaces inside the container so Docker port-mapping works.
# Override HOST=127.0.0.1 at runtime for local non-containerised dev.
ENV HOST=0.0.0.0
ENV PORT=8000

EXPOSE 8000

ENTRYPOINT ["scryr", "serve"]
