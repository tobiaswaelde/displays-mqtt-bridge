FROM rust:1.94-bookworm AS builder
WORKDIR /src
RUN apt-get update \
    && apt-get install -y --no-install-recommends libudev-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libudev1 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/mqtt-screen-control /usr/local/bin/mqtt-screen-control
USER 65534:65534
ENTRYPOINT ["/usr/local/bin/mqtt-screen-control"]
CMD ["--config", "/app/config/config.yml"]
