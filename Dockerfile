ARG APP_IMAGE_URL=cgr.dev/chainguard/wolfi-base
ARG APP_IMAGE_SHA=sha256:0e09bcd548cf2dfb9a3fd40af1a7389aa8c16b428de4e8f72b085f015694ce3d

# <== CHEF ==>
FROM lukemathwalker/cargo-chef:latest-rust-1.92.0 AS chef

WORKDIR /app

RUN set -eu; \
    apt update; \
    apt install lld clang -y --no-install-recommends;


# <== PLANNER ==>
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY src/lib.rs ./src/lib.rs
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json;


# <== BUILDER ==>
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
# Build all deps
RUN cargo chef cook --release --recipe-path recipe.json;

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release;

# <== RUNTIME ==>
FROM ${APP_IMAGE_URL}@${APP_IMAGE_SHA} AS runtime

COPY --from=builder /app/target/release/gh-dashboard gh-dashboard

ENTRYPOINT [ "./gh-dashboard" ]
