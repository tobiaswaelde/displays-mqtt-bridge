# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- A VitePress documentation site with deployment, hardware, MQTT, configuration,
  and troubleshooting guides.
- Project logo and favicon for the documentation site.

### Changed

- Renamed the project, Cargo package, Docker Compose service, container image,
  and configuration from `mqtt-screen-control` to `displays-mqtt-bridge`.
- **Breaking:** MQTT configuration is now nested below `mqtt`, with separate
  `protocol`, `host`, and `port` fields. The `base_topic` field is now `topic`.
- MQTT client IDs are optional; an empty or omitted `mqtt.client_id` now
  generates a UUID for the running process.

### Fixed

- Docker builds now include the patched `nom` dependency required by `ddc-hi`.
- Restored CI, documentation deployment, and release workflow compatibility
  with pnpm 11 and GitHub Pages.

## [0.1.0] - 2026-08-22

### Added

- MQTT-based DDC/CI control for the brightness of one or all detected displays.
- Retained MQTT value topics for brightness, contrast, RGB gain, input source,
  speaker volume, and power mode.
- Configurable periodic value refresh and structured, level-based logging.
- Docker Compose setup plus GitHub Actions workflows for validation and GHCR
  image publishing.
