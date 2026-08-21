use std::time::Duration;

use anyhow::Result;
use rumqttc::{AsyncClient, Event, Packet};
use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    time::MissedTickBehavior,
};
use tracing::{debug, error, info, warn};

use crate::{
    config::{Config, mqtt_options, update_interval},
    control::{ScanResult, ScreenController},
    mqtt::Publisher,
    protocol::{Command, CommandResult, DisplayStatus, Status},
};

pub async fn run(config: Config) -> Result<()> {
    let options = mqtt_options(&config)?;
    let update_interval = update_interval(&config);
    let (broker_host, broker_port) = options.broker_address();
    let (client, mut eventloop) = AsyncClient::new(options, 4096);
    let publisher = Publisher::new(client, &config.base_topic);
    publisher.subscribe_to_commands().await?;

    info!(%broker_host, broker_port, base_topic = %config.base_topic, update_interval_secs = config.update_interval_secs, "starting MQTT screen controller");
    let (scan_sender, mut scan_receiver) = unbounded_channel();
    let mut controller = Some(ScreenController::new());
    let mut last_displays = vec![];
    let mut update_timer = tokio::time::interval(update_interval);
    // Tokio's first interval tick is immediate; consume it because connection setup starts the first scan.
    update_timer.tick().await;
    update_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            event = eventloop.poll() => match event {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    info!("MQTT connection established");
                    // Publish an immediate retained heartbeat while slow I2C probing happens in the worker.
                    publish_status(&publisher, &last_displays, None).await;
                    if let Some(controller) = controller.take() {
                        start_scan(controller, true, None, scan_sender.clone());
                    } else {
                        debug!("DDC/CI scan is already running after MQTT reconnect");
                    }
                }
                Ok(Event::Incoming(Packet::Publish(message))) if message.topic == publisher.command_topic() => {
                    info!(topic = %message.topic, bytes = message.payload.len(), "received MQTT command");
                    let result = handle_command(&message.payload, controller.take());
                    controller = result.controller;
                    if let Some((next_controller, refresh, command)) = result.start_scan {
                        start_scan(next_controller, refresh, Some(command), scan_sender.clone());
                    } else if let Some(command) = result.immediate_response {
                        publish_status(&publisher, &last_displays, Some(command)).await;
                    }
                }
                Ok(Event::Incoming(Packet::Publish(message))) => {
                    debug!(topic = %message.topic, "ignoring publish from an unsubscribed topic");
                }
                Ok(event) => debug!(?event, "processed MQTT event"),
                Err(error) => {
                    error!(%error, "MQTT event loop failed; retrying in two seconds");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            },
            Some(scan) = scan_receiver.recv() => {
                last_displays = scan.displays.clone();
                controller = Some(scan.controller);
                publish_status(&publisher, &last_displays, scan.command).await;
                if let Err(error) = publisher.publish_details(scan.publications).await {
                    warn!(%error, "could not publish DDC/CI display details");
                }
            },
            _ = update_timer.tick() => {
                if let Some(controller) = controller.take() {
                    debug!("starting scheduled DDC/CI value update");
                    start_scan(controller, false, None, scan_sender.clone());
                } else {
                    debug!("skipping scheduled DDC/CI update because a scan is already running");
                }
            }
        }
    }
}

struct CommandHandling {
    controller: Option<ScreenController>,
    start_scan: Option<(ScreenController, bool, CommandResult)>,
    immediate_response: Option<CommandResult>,
}

fn handle_command(payload: &[u8], controller: Option<ScreenController>) -> CommandHandling {
    let command = match serde_json::from_slice::<Command>(payload) {
        Ok(command) => command,
        Err(error) => {
            warn!(%error, "received invalid MQTT command JSON");
            return CommandHandling {
                controller,
                start_scan: None,
                immediate_response: Some(CommandResult::failure(format!(
                    "invalid command JSON: {error}"
                ))),
            };
        }
    };

    let Some(mut controller) = controller else {
        warn!("rejecting MQTT command while DDC/CI scan is in progress");
        return CommandHandling {
            controller: None,
            start_scan: None,
            immediate_response: Some(CommandResult::failure(
                "DDC/CI scan already in progress; retry shortly",
            )),
        };
    };

    let (refresh, result) = match command {
        Command::ListDisplays => {
            info!("refreshing displays on list_displays command");
            (true, CommandResult::success())
        }
        Command::ScanDisplays => {
            info!("refreshing configured DDC/CI values");
            (false, CommandResult::success())
        }
        Command::SetBrightness {
            display,
            brightness,
        } => (false, controller.set_brightness(&display, brightness)),
    };

    // The DDC/CI scan runs on a blocking worker, leaving this task free to service MQTT keep-alives.
    CommandHandling {
        controller: None,
        start_scan: Some((controller, refresh, result)),
        immediate_response: None,
    }
}

fn start_scan(
    controller: ScreenController,
    refresh: bool,
    command: Option<CommandResult>,
    sender: UnboundedSender<ScanResult>,
) {
    tokio::task::spawn_blocking(move || {
        debug!(refresh, "starting blocking DDC/CI scan worker");
        let result = controller.scan(refresh, command);
        if sender.send(result).is_err() {
            warn!("could not return DDC/CI scan result because MQTT task stopped");
        }
    });
}

async fn publish_status(
    publisher: &Publisher,
    displays: &[DisplayStatus],
    command: Option<CommandResult>,
) {
    let status = Status {
        displays: displays.to_vec(),
        command,
    };
    if let Err(error) = publisher.publish_status(&status).await {
        warn!(%error, "could not publish aggregate status");
    }
}
