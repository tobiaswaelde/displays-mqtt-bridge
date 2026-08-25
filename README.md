<!-- omit in toc -->
# 🖥️ Displays MQTT Bridge

[![CI](https://img.shields.io/github/actions/workflow/status/tobiaswaelde/displays-mqtt-bridge/ci.yml?style=for-the-badge&label=CI)](https://github.com/tobiaswaelde/displays-mqtt-bridge/actions/workflows/ci.yml) [![Docs](https://img.shields.io/github/actions/workflow/status/tobiaswaelde/displays-mqtt-bridge/docs.yml?style=for-the-badge&label=Docs)](https://github.com/tobiaswaelde/displays-mqtt-bridge/actions/workflows/docs.yml) [![Deploy](https://img.shields.io/github/actions/workflow/status/tobiaswaelde/displays-mqtt-bridge/deploy.yml?style=for-the-badge&label=Deploy)](https://github.com/tobiaswaelde/displays-mqtt-bridge/actions/workflows/deploy.yml)

[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-tobiaswaelde-FFDD00?style=for-the-badge&logo=buymeacoffee)](https://www.buymeacoffee.com/tobiaswaelde)

A small Rust service that controls DDC/CI-capable external monitors through MQTT. It detects every monitor available through the mounted `/dev/i2c-*` devices and can set the brightness of an individual display or every display at once.

Full documentation is available at [tobiaswaelde.github.io/displays-mqtt-bridge](https://tobiaswaelde.github.io/displays-mqtt-bridge/).

- [🚀 Quick start](#-quick-start)
- [🐳 Docker](#-docker)
- [🧩 Compose example](#-compose-example)
- [🦀 Run without Docker](#-run-without-docker)
- [⚙ Configuration](#-configuration)
- [📡 Logging](#-logging)
- [🎛 Commands](#-commands)
- [📝 Notes](#-notes)
- [🔄 CI and deployment](#-ci-and-deployment)

## 🚀 Quick start

```sh
cp config/config.example.yml config/config.yml
# Edit the broker connection, credentials, topic, and update interval.
$EDITOR config/config.yml
docker compose up --build -d
```

Use `docker compose logs -f` to follow the service and `docker compose down` to stop it.

## 🐳 Docker

The included [`compose.yml`](compose.yml) builds the image locally. Adjust its `/dev/i2c-*` mappings and the `i2c` group GID for the host before starting it.

The container needs access to every host I²C adapter that is connected to a monitor. Discover them on the host with `i2cdetect -l`. This checkout maps the host's NVIDIA adapters `/dev/i2c-3` through `/dev/i2c-6`; adjust the list for another host. The Docker user must also have permission to access the mapped devices; set `group_add` to the numeric GID of the host `i2c` group. Some displays, docking stations, HDMI/DisplayPort adapters, and laptop panels do not expose DDC/CI.

## 🧩 Compose example

[`compose.example.yml`](compose.example.yml) runs the published GHCR image instead of building it locally:

```sh
cp compose.example.yml compose.yml
cp config/config.example.yml config/config.yml
# Adjust config/config.yml, I²C adapters, and i2c group GID.
docker compose pull
docker compose up -d
```

The example uses `/dev/i2c-3` through `/dev/i2c-6` and GID `967`, matching the documented NVIDIA setup. Replace both values on other hosts.

## 🦀 Run without Docker

Install a Rust toolchain and ensure the current user can access the required I²C devices (typically by being in the `i2c` group). Then run:

```sh
cargo run --release -- --config config/config.yml
```

The application uses the same configuration and MQTT topics in both modes.

## ⚙ Configuration

`config/config.yml`:

```yaml
mqtt:
  protocol: mqtt
  host: mosquitto
  port: 1883
  client_id: ''
  username: screen-control
  password: change-me
topic: screens/office
update_interval_secs: 60
```

`client_id` is optional. When it is omitted or empty, the bridge generates a UUID for the running process.

The service subscribes to `<topic>/cmd` with QoS 1. It publishes an initial and command-result status document, retained, to `<topic>/status`.

Every detected display additionally publishes retained DDC/CI data below `<topic>/displays/<index>/`. `identity` contains EDID identity data. The following VCP features are intentionally limited to `brightness`, `contrast`, `red_gain`, `green_gain`, `blue_gain`, `input_source`, `speaker_volume`, and `power_mode`.

Each feature has three direct subtopics, for example:

```text
screens/office/displays/0/brightness/value       -> 42
screens/office/displays/0/brightness/maximum     -> 100
screens/office/displays/0/brightness/value_type  -> 0
```

The payloads are JSON numbers. If a monitor cannot read a feature, all three corresponding retained payloads are `null`; this deliberately replaces any stale value. VCP codes, percentages, raw capability strings, and vendor-specific features are not published.

At startup, after every accepted control or refresh command, and at the configured `update_interval_secs`, the selected values are refreshed. The service publishes an empty retained status as soon as the broker connection is established, then performs its I²C scan in the background. All status topics are retained.

## 📡 Logging

The default level is `info`. Control it with `RUST_LOG`; messages include MQTT connection state, display discovery, DDC/CI capability scans, command handling, and per-display errors.

```sh
# Detailed MQTT and DDC/CI diagnostics without Docker
RUST_LOG=displays_mqtt_bridge=debug cargo run --release -- --config config/config.yml

# Include every individual VCP read (very verbose)
RUST_LOG=displays_mqtt_bridge=trace cargo run --release -- --config config/config.yml

# Use the same level with Docker Compose
RUST_LOG=debug docker compose up
```

## 🎛 Commands

Set all detected displays to 60%:

```sh
mosquitto_pub -t screens/office/cmd -m \
  '{"command":"set_brightness","display":"all","brightness":60}'
```

Set one display by its zero-based index (use `list_displays` first):

```sh
mosquitto_pub -t screens/office/cmd -m \
  '{"command":"set_brightness","display":0,"brightness":35}'
```

Refresh device detection and return the available displays:

```sh
mosquitto_pub -t screens/office/cmd -m '{"command":"list_displays"}'
mosquitto_sub -t screens/office/status -C 1
```

Immediately refresh the configured feature values without rediscovering displays:

```sh
mosquitto_pub -t screens/office/cmd -m '{"command":"scan_displays"}'
```

`brightness` must be an integer from 0 to 100. A `set_brightness` reply contains each selected display, the requested percentage, and either its applied VCP value or an error; failure of one display does not prevent attempting the others. After the command, all configured feature topics are refreshed.

## 📝 Notes

Brightness is set through the standard MCCS VCP feature `0x10` (luminance). The percentage is scaled to each monitor's reported maximum, so a `brightness` value has the same intended meaning across displays with different VCP ranges.

## 🔄 CI and deployment

GitHub Actions runs formatting, Clippy, tests, a release build, and a Docker image build for pull requests and pushes to `main`. Pushing a version tag such as `v1.2.3` builds and publishes the image to GitHub Container Registry under `ghcr.io/<owner>/displays-mqtt-bridge` with the tags `1.2.3`, `1.2`, and `latest`. The deployment workflow can also be started manually and requires an additional image tag.
