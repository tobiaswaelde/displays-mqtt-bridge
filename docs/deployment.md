# Docker deployment

The container runs as the unprivileged UID and GID `65534`. It needs read/write access to the host I²C adapters that expose DDC/CI for the target displays.

## Build locally

Create a local, ignored Compose file and configuration:

```sh
cp compose.example.yml compose.yml
cp config/config.example.yml config/config.yml
# Edit config/config.yml, compose.yml, adapter mappings, and group ID.
docker compose up --build -d
```

The configuration directory is mounted read-only at `/app/config`. Changes to `config/config.yml` require a container restart.

## Use the published image

`compose.example.yml` uses `ghcr.io/tobiaswaelde/displays-mqtt-bridge:latest`. For a reproducible deployment, replace `latest` with a release tag before starting:

```sh
docker compose pull
docker compose up -d
```

## Device permissions

1. On the host, list the available adapters:

   ```sh
   i2cdetect -l
   ```

2. Map only the adapters connected to the displays you want to control, for example:

   ```yaml
   devices:
     - /dev/i2c-3:/dev/i2c-3
   ```

3. Get the host's numeric `i2c` group ID and set it under `group_add`:

   ```sh
   getent group i2c
   ```

   ```yaml
   group_add:
     - "967"
   ```

`967` is only an example. Use the numeric value from the deployment host. Do not use privileged mode or map every device merely to work around an incorrect adapter or group configuration.

See [hardware requirements](/hardware) for DDC/CI limitations and diagnostic steps.
