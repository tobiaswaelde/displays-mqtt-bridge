# Troubleshooting

## No displays appear

1. Run `i2cdetect -l` on the host and verify that the matching `/dev/i2c-*` entries are mapped in Compose.
2. Compare `getent group i2c` with the numeric `group_add` value in Compose.
3. Subscribe to `<topic>/status`; it should first publish `{"displays":[]}` and then the discovered displays.
4. Check that the display path supports DDC/CI. A dock, cable, graphics adapter, or laptop panel can prevent DDC/CI even when a display works normally.

## MQTT cannot connect

- `mqtt.host` is only a hostname or IP address; do not include `mqtt://`, a port, or a path.
- Set `mqtt.port` explicitly. TLS listeners commonly use `8883`.
- Configure both `mqtt.username` and `mqtt.password`, or omit both.
- For `mqtts`/`ssl`, ensure the broker certificate is trusted by the container or runtime.
- Use `RUST_LOG=debug` and inspect `docker compose logs -f` for connection errors.

## A command has no expected effect

- Publish non-retained JSON to `<topic>/cmd`; retained commands can run again after reconnect.
- Use `list_displays` and the returned zero-based `index` before selecting one display.
- Keep `brightness` between `0` and `100`.
- Wait for the retained `<topic>/status` response. A DDC/CI scan in progress returns a retryable error.

## A value is `null`

`null` is an intentional retained payload. It means that the display could not read that VCP feature, and it replaces any older value. It does not mean that the MQTT connection failed.

## Safe diagnostics

Use `RUST_LOG=debug` first; use `trace` only for detailed VCP reads. Do not include MQTT passwords, EDID serial data, or host-specific device mappings in issue reports.
