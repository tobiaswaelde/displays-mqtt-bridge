FROM rust:1.94-bookworm AS builder
WORKDIR /src
RUN apt-get update \
    && apt-get install -y --no-install-recommends libudev-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY third_party ./third_party
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libudev1 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/displays-mqtt-bridge /usr/local/bin/displays-mqtt-bridge
USER 65534:65534
ENTRYPOINT ["/usr/local/bin/displays-mqtt-bridge"]
CMD ["--config", "/app/config/config.yml"]
