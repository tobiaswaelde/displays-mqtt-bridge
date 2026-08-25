# Getting started

1. Copy `config/config.example.yml` to `config/config.yml`.
2. Configure the MQTT broker, credentials, and topic.
3. Map each DDC/CI-capable `/dev/i2c-*` adapter and the host `i2c` group in `compose.yml`.
4. Start the bridge with `docker compose up -d`.

Use `i2cdetect -l` on the host to find adapters. A monitor, dock, cable, or adapter that does not expose DDC/CI cannot be controlled by this bridge.
