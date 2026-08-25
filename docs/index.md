---
layout: home

hero:
  name: Displays MQTT Bridge
  text: DDC/CI display control through MQTT
  tagline: Discover connected monitors, publish their selected values, and control brightness through one predictable MQTT contract.
  image:
    src: /logo.svg
    alt: Displays MQTT Bridge logo
  actions:
    - theme: brand
      text: Get started
      link: /getting-started
    - theme: alt
      text: MQTT contract
      link: /mqtt
    - theme: alt
      text: GitHub
      link: https://github.com/tobiaswaelde/displays-mqtt-bridge

features:
  - icon: 🖥️
    title: DDC/CI-aware discovery
    details: Finds displays exposed through the configured I²C adapters and publishes their identity and selected VCP values.
  - icon: ⚡
    title: Direct MQTT control
    details: Set one display or every discovered display to a requested brightness percentage with a compact JSON command.
  - icon: 🔒
    title: Explicit operations contract
    details: Use retained status and value topics, non-sensitive local configuration, and a deliberate allowlist of monitor features.
---

## Built for local display automation

The bridge connects DDC/CI-capable external displays to your broker without exposing arbitrary monitor controls. It publishes brightness, contrast, color gains, input source, speaker volume, and power mode below the configured topic.

Start with the [configuration guide](/configuration), then use the [MQTT contract](/mqtt) to automate your displays.

> DDC/CI support depends on the display, dock, cable, and adapter. Confirm the correct host adapters with `i2cdetect -l` before deploying.
