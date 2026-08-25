# Configuration

Copy `config/config.example.yml` to `config/config.yml`. The default command-line argument and the container both read `/app/config/config.yml`; use `--config <path>` only when running the binary with a different file location.

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

Keep `config/config.yml` local. It can contain broker credentials and is intentionally ignored by Git.

## MQTT connection

| Key | Required | Default | Description |
| --- | --- | --- | --- |
| `mqtt.protocol` | No | `mqtt` | `mqtt` or its alias `tcp` use an unencrypted connection. `mqtts` or its alias `ssl` enable TLS. |
| `mqtt.host` | Yes | — | Broker hostname or IP address, without a URL scheme or port. |
| `mqtt.port` | No | `1883` | Broker TCP port. Set `8883` explicitly for a conventional TLS listener. |
| `mqtt.client_id` | No | random UUID | A missing, empty, or whitespace-only value creates a UUID v4 for that process. Set a stable value when the broker requires a persistent session. |
| `mqtt.username` | No | — | MQTT user name. It must be set together with `mqtt.password`. |
| `mqtt.password` | No | — | MQTT password. It must be set together with `mqtt.username`. |

For `mqtts`/`ssl`, the broker certificate must be trusted by the runtime. The bridge does not provide configuration for a custom CA, client certificate, or insecure certificate validation.

## Topic and refresh interval

| Key | Required | Default | Description |
| --- | --- | --- | --- |
| `topic` | Yes | — | Root for every MQTT topic. Leading and trailing slashes are removed before publishing or subscribing, and the remaining value must not be empty. |
| `update_interval_secs` | No | `60` | Seconds between value refreshes. It must be greater than zero. |

For example, `topic: screens/office` produces `screens/office/cmd`, `screens/office/status`, and `screens/office/displays/#`.

## Migrating the previous configuration

The previous flat configuration is not supported. Move its broker URL components under `mqtt`, and rename `base_topic` to `topic`:

```yaml
# Before
broker: mqtt://mosquitto:1883
client_id: screen-control
base_topic: screens/office

# After
mqtt:
  protocol: mqtt
  host: mosquitto
  port: 1883
  client_id: screen-control
topic: screens/office
```

Move `username` and `password` into the same `mqtt` block as well.
