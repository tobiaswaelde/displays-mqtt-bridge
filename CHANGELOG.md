# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-22

### Added

- MQTT-based DDC/CI control for the brightness of one or all detected displays.
- Retained MQTT value topics for brightness, contrast, RGB gain, input source,
  speaker volume, and power mode.
- Configurable periodic value refresh and structured, level-based logging.
- Docker Compose setup plus GitHub Actions workflows for validation and GHCR
  image publishing.
