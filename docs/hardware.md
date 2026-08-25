# Hardware requirements

The bridge controls external displays through Linux I²C adapters and DDC/CI. A successfully mapped `/dev/i2c-*` device is necessary but does not guarantee that a display supports every VCP feature.

## Before deploying

- List host adapters with `i2cdetect -l`.
- Map only adapters belonging to the intended displays.
- Ensure the container has the host `i2c` group ID through `group_add`.
- Confirm that the display, dock, cable, and video adapter expose DDC/CI. Laptop panels and some docks or adapters commonly do not.

## Verify discovery

Start the bridge and subscribe to the retained state:

```sh
mosquitto_sub -v -t 'screens/office/status' -t 'screens/office/displays/#'
```

Replace `screens/office` with the configured topic. The status topic first contains an empty display list while discovery runs, then publishes the detected displays. If discovery is empty, check the adapter mappings and host permissions before changing MQTT settings.

## Feature availability

Displays differ in the VCP features they implement. An unreadable configured feature is not an application failure: its `value`, `maximum`, and `value_type` topics are retained as JSON `null`. Brightness control requires the standard luminance VCP feature `0x10`.

Use `RUST_LOG=debug` for discovery and command diagnostics. `RUST_LOG=trace` includes individual VCP reads and should be used only while investigating a device.
