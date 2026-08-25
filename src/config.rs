use std::{fs, path::Path, time::Duration};

use anyhow::{Context, Result, bail};
use rumqttc::{MqttOptions, TlsConfiguration, Transport};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub topic: String,
    #[serde(default = "default_update_interval_secs")]
    pub update_interval_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct MqttConfig {
    #[serde(default = "default_mqtt_protocol")]
    pub protocol: String,
    pub host: String,
    #[serde(default = "default_mqtt_port")]
    pub port: u16,
    #[serde(default)]
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

const fn default_update_interval_secs() -> u64 {
    60
}

fn default_mqtt_protocol() -> String {
    "mqtt".to_owned()
}

const fn default_mqtt_port() -> u16 {
    1883
}

pub fn load_config(path: &Path) -> Result<Config> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("could not read configuration {}", path.display()))?;
    let config: Config = serde_yaml::from_str(&raw).context("invalid YAML configuration")?;
    if config.topic.trim_matches('/').is_empty() {
        bail!("topic must not be empty");
    }
    if config.update_interval_secs == 0 {
        bail!("update_interval_secs must be greater than zero");
    }
    Ok(config)
}

pub fn update_interval(config: &Config) -> Duration {
    Duration::from_secs(config.update_interval_secs)
}

pub fn mqtt_options(config: &Config) -> Result<MqttOptions> {
    let tls = match config.mqtt.protocol.as_str() {
        "mqtt" | "tcp" => false,
        "mqtts" | "ssl" => true,
        protocol => bail!("unsupported MQTT protocol {protocol:?}; use mqtt or mqtts"),
    };

    let client_id = config
        .mqtt
        .client_id
        .as_deref()
        .filter(|client_id| !client_id.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut options = MqttOptions::new(client_id, &config.mqtt.host, config.mqtt.port);
    if tls {
        options.set_transport(Transport::tls_with_config(TlsConfiguration::Native));
    }
    options.set_keep_alive(Duration::from_secs(30));

    match (&config.mqtt.username, &config.mqtt.password) {
        (Some(username), Some(password)) => options.set_credentials(username, password),
        (None, None) => &mut options,
        _ => bail!("username and password must either both be set or both be omitted"),
    };

    Ok(options)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Config, MqttConfig, mqtt_options, update_interval};
    use uuid::Uuid;

    #[test]
    fn deserializes_the_nested_mqtt_configuration() {
        let config: Config = serde_yaml::from_str(
            r#"
mqtt:
  host: broker.example.test
topic: screens/office
"#,
        )
        .unwrap();

        assert_eq!(config.mqtt.protocol, "mqtt");
        assert_eq!(config.mqtt.port, 1883);
        assert_eq!(config.mqtt.client_id, None);
        assert_eq!(config.topic, "screens/office");
        assert_eq!(config.update_interval_secs, 60);
    }

    #[test]
    fn mqtt_configuration_uses_the_configured_client_id() {
        let config = Config {
            mqtt: MqttConfig {
                protocol: "mqtt".to_owned(),
                host: "broker.example.test".to_owned(),
                port: 1883,
                client_id: Some("screen-controller".to_owned()),
                username: None,
                password: None,
            },
            topic: "screens/office".to_owned(),
            update_interval_secs: 60,
        };

        let options = mqtt_options(&config).unwrap();
        assert_eq!(options.client_id(), "screen-controller");
        assert_eq!(
            options.broker_address(),
            ("broker.example.test".to_owned(), 1883)
        );
        assert_eq!(update_interval(&config), Duration::from_secs(60));
    }

    #[test]
    fn mqtt_configuration_generates_a_client_id_when_omitted() {
        let config = Config {
            mqtt: MqttConfig {
                protocol: "mqtt".to_owned(),
                host: "broker.example.test".to_owned(),
                port: 1883,
                client_id: None,
                username: None,
                password: None,
            },
            topic: "screens/office".to_owned(),
            update_interval_secs: 60,
        };

        let options = mqtt_options(&config).unwrap();
        assert!(Uuid::parse_str(&options.client_id()).is_ok());
    }
}
