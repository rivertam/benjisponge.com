# Build context is the repo root (see wrangler.jsonc image_build_context / Justfile).

FROM rust:1.97-slim AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install topcoat-cli --version 0.1.3 --locked
WORKDIR /build
COPY Cargo.toml Cargo.lock build.rs .gitignore ./
COPY src ./src
COPY styles ./styles
COPY data ./data
# build.rs downloads the standalone Tailwind v4 CLI here (needs network + certs)
RUN cargo build --release
RUN topcoat asset bundle --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/target/release/benjisponge /app/benjisponge
# AssetBundle::load() walks up from the exe looking for assets/manifest.toml
COPY --from=builder /build/target/assets /app/assets
ENV HOST=0.0.0.0 PORT=8080
EXPOSE 8080
CMD ["/app/benjisponge"]
