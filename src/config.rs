use std::{fs, path::Path, time::Duration};

use anyhow::{Context, Result, bail};
use rumqttc::{MqttOptions, Transport};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub broker: String,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub base_topic: String,
    #[serde(default = "default_update_interval_secs")]
    pub update_interval_secs: u64,
}

const fn default_update_interval_secs() -> u64 {
    60
}

pub fn load_config(path: &Path) -> Result<Config> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("could not read configuration {}", path.display()))?;
    let config: Config = serde_yaml::from_str(&raw).context("invalid YAML configuration")?;
    if config.base_topic.trim_matches('/').is_empty() {
        bail!("base_topic must not be empty");
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
    let broker = Url::parse(&config.broker).context("invalid MQTT broker URL")?;
    let host = broker
        .host_str()
        .context("MQTT broker URL must include a host")?;
    let (port, tls) = match broker.scheme() {
        "mqtt" | "tcp" => (broker.port().unwrap_or(1883), false),
        "mqtts" | "ssl" => (broker.port().unwrap_or(8883), true),
        scheme => bail!("unsupported MQTT broker URL scheme {scheme:?}; use mqtt:// or mqtts://"),
    };

    // The YAML client ID is authoritative; rumqttc's URL helper requires it in the URL.
    let mut options = MqttOptions::new(&config.client_id, host, port);
    if tls {
        options.set_transport(Transport::tls_with_default_config());
    }
    options.set_keep_alive(Duration::from_secs(30));

    match (&config.username, &config.password) {
        (Some(username), Some(password)) => options.set_credentials(username, password),
        (None, None) => &mut options,
        _ => bail!("username and password must either both be set or both be omitted"),
    };

    Ok(options)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Config, mqtt_options, update_interval};

    #[test]
    fn broker_url_uses_client_id_from_config() {
        let config = Config {
            broker: "mqtt://broker.example.test:1883".to_owned(),
            client_id: "screen-controller".to_owned(),
            username: None,
            password: None,
            base_topic: "screens/office".to_owned(),
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
}
