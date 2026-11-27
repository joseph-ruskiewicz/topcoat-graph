# Builds the docs site (the `site` package) and bundles its assets next to the binary.
#
# The dependencies are built in a layer of their own (cargo-chef), which Docker reuses
# until Cargo.toml or Cargo.lock change, so most builds compile only this workspace.
FROM rust:1-slim-bookworm AS chef
RUN cargo install cargo-chef --version 0.1.78 --locked
RUN cargo install topcoat-cli --version 0.12.1 --locked
WORKDIR /app

# The dependency list, read from the manifests alone.
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --package site --recipe-path recipe.json
COPY . .
RUN topcoat asset bundle --release --package site

# Run: the binary with its assets beside it. The site makes no outbound requests, so
# the image needs no CA certificates.
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /app/target/release/site /app/bin/site
COPY --from=build /app/target/release/assets /app/bin/assets
ENV HOST=0.0.0.0 \
    PORT=8080
EXPOSE 8080
CMD ["/app/bin/site"]
