use anyhow::{Context, Result};
use rumqttc::{AsyncClient, QoS};
use serde::Serialize;
use tracing::{debug, trace};

use crate::protocol::{Publication, Status};

#[derive(Clone)]
pub struct Publisher {
    client: AsyncClient,
    topic: String,
}

impl Publisher {
    pub fn new(client: AsyncClient, topic: &str) -> Self {
        Self {
            client,
            topic: topic.trim_matches('/').to_owned(),
        }
    }

    pub fn command_topic(&self) -> String {
        format!("{}/cmd", self.topic)
    }

    pub async fn subscribe_to_commands(&self) -> Result<()> {
        let topic = self.command_topic();
        debug!(%topic, "subscribing to MQTT command topic");
        self.client
            .subscribe(topic, QoS::AtLeastOnce)
            .await
            .context("could not subscribe to command topic")
    }

    pub async fn publish_status(&self, status: &Status) -> Result<()> {
        self.publish("status", status).await
    }

    pub async fn publish_details(&self, publications: Vec<Publication>) -> Result<()> {
        for publication in publications {
            self.publish(&publication.topic_suffix, &publication.payload)
                .await?;
        }
        Ok(())
    }

    async fn publish<T: Serialize>(&self, suffix: &str, value: &T) -> Result<()> {
        let topic = format!("{}/{}", self.topic, suffix);
        let payload = serde_json::to_vec(value).context("could not serialize MQTT payload")?;
        trace!(%topic, bytes = payload.len(), "publishing retained MQTT payload");
        self.client
            .publish(&topic, QoS::AtLeastOnce, true, payload)
            .await
            .with_context(|| format!("could not publish MQTT topic {topic}"))
    }
}
