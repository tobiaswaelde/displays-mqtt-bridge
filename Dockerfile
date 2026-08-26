# =============================================================================
# Build stage
# =============================================================================

# Compile the Rust binary in a disposable build stage.
FROM rust:1.97-bookworm AS builder
WORKDIR /src

# -----------------------------------------------------------------------------
# Native build dependencies
# -----------------------------------------------------------------------------

# Install only the native libraries and tools required by the DDC/CI build.
RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl-dev libudev-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

# -----------------------------------------------------------------------------
# Application build
# -----------------------------------------------------------------------------

# Copy manifests before sources to keep dependency compilation cacheable.
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY third_party ./third_party
RUN cargo build --release

# =============================================================================
# Runtime stage
# =============================================================================

# Keep the runtime image limited to the binary and its native runtime libraries.
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 libudev1 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/displays-mqtt-bridge /usr/local/bin/displays-mqtt-bridge

# -----------------------------------------------------------------------------
# Container health and execution
# -----------------------------------------------------------------------------

# The bridge owns PID 1; verify that its process entry is available while the container runs.
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 CMD ["test", "-r", "/proc/1/cmdline"]

# DDC/CI access is supplied through explicitly mapped host adapters, not privileged mode.
USER 65534:65534
ENTRYPOINT ["/usr/local/bin/displays-mqtt-bridge"]
CMD ["--config", "/app/config/config.yml"]
