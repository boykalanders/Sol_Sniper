# --- build stage ---
FROM rust:1.79-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

# --- runtime stage ---
FROM debian:bookworm-slim
WORKDIR /bot
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/snipe /usr/local/bin/snipe
COPY config.toml .FROM rust:1.79-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /bot
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/snipe /usr/local/bin/snipe
COPY config.toml .
COPY keys keys/
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/snipe"]
COPY keys/id.json keys/
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/snipe"]