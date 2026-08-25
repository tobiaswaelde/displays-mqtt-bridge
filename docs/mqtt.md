# MQTT contract

Replace `<topic>` below with the configured `topic` value. The bridge subscribes and publishes with QoS 1. Every topic published by the bridge is retained, so a newly connected subscriber immediately receives the most recent state.

| Topic | Direction | Payload | Retained |
| --- | --- | --- | --- |
| `<topic>/cmd` | Client → bridge | JSON command | No — commands must not be retained. |
| `<topic>/status` | Bridge → client | Aggregate discovery and command result JSON | Yes |
| `<topic>/displays/<index>/identity` | Bridge → client | Display identity JSON | Yes |
| `<topic>/displays/<index>/<feature>/{value,maximum,value_type}` | Bridge → client | JSON number or `null` | Yes |

Do not retain a command. A retained command can be delivered again when the bridge reconnects and may repeat a control action.

## Commands

Send commands to `<topic>/cmd` as JSON.

### Set brightness

`display` is either `"all"` or a zero-based display index. `brightness` must be an integer from `0` through `100`.

```json
{
  "command": "set_brightness",
  "display": "all",
  "brightness": 60
}
```

```json
{
  "command": "set_brightness",
  "display": 0,
  "brightness": 35
}
```

The requested percentage is converted to the selected display's reported VCP luminance range. The bridge tries every selected display; one failure does not stop the other updates.

### Rediscover displays

`list_displays` runs DDC/CI discovery again, then publishes the refreshed status and values.

```json
{
  "command": "list_displays"
}
```

### Refresh values

`scan_displays` keeps the current display list and refreshes the configured VCP values.

```json
{
  "command": "scan_displays"
}
```

Only one DDC/CI scan runs at a time. A command received while one is running gets an error result and can be retried shortly.

## Status responses

When the process first connects, the bridge immediately publishes this retained heartbeat while the first DDC/CI discovery runs:

```json
{
  "displays": []
}
```

After a reconnect, it republishes the most recently discovered display list before starting another discovery scan.

After discovery, `<topic>/status` lists the available displays:

```json
{
  "displays": [
    {
      "index": 0,
      "id": "display-identifier",
      "name": "Office Display"
    }
  ]
}
```

A completed brightness command adds `command`. `applied_vcp_value` is the raw luminance value written to that particular monitor.

```json
{
  "displays": [
    {
      "index": 0,
      "id": "display-identifier",
      "name": "Office Display"
    }
  ],
  "command": {
    "ok": true,
    "results": [
      {
        "index": 0,
        "id": "display-identifier",
        "name": "Office Display",
        "requested_percent": 60,
        "applied_vcp_value": 60
      }
    ]
  }
}
```

Invalid JSON, invalid targets, and busy scans are reported with `ok: false` and an `error` string:

```json
{
  "displays": [],
  "command": {
    "ok": false,
    "error": "DDC/CI scan already in progress; retry shortly"
  }
}
```

When a brightness write fails for only some displays, `ok` is `false`; each affected result has its own `error` and omits `applied_vcp_value`.

## Display topics

Each detected display uses its zero-based index in the topic. Its `identity` document includes backend and EDID-derived metadata such as manufacturer, model, serial, and MCCS version when the display provides them:

```json
{
  "backend": "i2c",
  "id": "display-identifier",
  "manufacturer_id": "DEL",
  "model_id": 1234,
  "version": "1.3",
  "serial": 123456,
  "manufacture_year": 2024,
  "manufacture_week": 10,
  "model_name": "Office Display",
  "serial_number": "ABC123",
  "mccs_version": "2.2"
}
```

Fields with unavailable EDID data are `null`; `edid_hex` is omitted when raw EDID bytes are unavailable. Restrict MQTT read access if this metadata is sensitive in your environment.

The bridge exposes only these standard VCP features:

`brightness`, `contrast`, `red_gain`, `green_gain`, `blue_gain`, `input_source`, `speaker_volume`, and `power_mode`.

For every feature, it publishes three JSON-number topics:

```text
<topic>/displays/0/brightness/value       -> 42
<topic>/displays/0/brightness/maximum     -> 100
<topic>/displays/0/brightness/value_type  -> 0
```

If a display cannot read a feature, all three retained payloads become JSON `null`. This deliberately replaces stale values; the bridge does not publish arbitrary VCP codes, percentage conversions, capability strings, or vendor-specific features.
