use anyhow::{Context, Result, bail};
use ddc_hi::{Ddc, Display, FeatureCode};
use serde::Serialize;
use serde_json::json;
use tracing::{debug, info, trace, warn};

use crate::protocol::{BrightnessResult, CommandResult, DisplayStatus, DisplayTarget, Publication};

const BRIGHTNESS: FeatureCode = 0x10;

/// The intentionally small set of standard VCP features exposed via MQTT.
const FEATURES: &[Feature] = &[
    Feature::new("brightness", BRIGHTNESS),
    Feature::new("contrast", 0x12),
    Feature::new("red_gain", 0x16),
    Feature::new("green_gain", 0x18),
    Feature::new("blue_gain", 0x1a),
    Feature::new("input_source", 0x60),
    Feature::new("speaker_volume", 0x62),
    Feature::new("power_mode", 0xd6),
];

#[derive(Copy, Clone)]
struct Feature {
    topic: &'static str,
    code: FeatureCode,
}

impl Feature {
    const fn new(topic: &'static str, code: FeatureCode) -> Self {
        Self { topic, code }
    }
}

pub struct ScreenController {
    displays: Vec<Display>,
}

pub struct ScanResult {
    pub controller: ScreenController,
    pub displays: Vec<DisplayStatus>,
    pub publications: Vec<Publication>,
    pub command: Option<CommandResult>,
}

impl ScreenController {
    pub fn new() -> Self {
        Self { displays: vec![] }
    }

    pub fn refresh(&mut self) {
        info!("starting DDC/CI display discovery");
        self.displays = Display::enumerate();
        info!(
            count = self.displays.len(),
            "DDC/CI display discovery complete"
        );

        for (index, screen) in self.displays.iter().enumerate() {
            debug!(
                display_index = index,
                id = %screen.info.id,
                model = ?screen.info.model_name,
                backend = %screen.info.backend,
                "detected DDC/CI display"
            );
        }
    }

    pub fn status(&self) -> Vec<DisplayStatus> {
        self.displays
            .iter()
            .enumerate()
            .map(|(index, display)| DisplayStatus {
                index,
                id: display.info.id.clone(),
                name: display
                    .info
                    .model_name
                    .clone()
                    .unwrap_or_else(|| "Unknown display".to_owned()),
            })
            .collect()
    }

    pub fn set_brightness(&mut self, target: &DisplayTarget, brightness: u8) -> CommandResult {
        if brightness > 100 {
            return CommandResult::failure("brightness must be an integer from 0 to 100");
        }

        let indices = match target.indices(self.displays.len()) {
            Ok(indices) => indices,
            Err(error) => return CommandResult::failure(error),
        };

        let results = indices
            .into_iter()
            .map(|index| self.set_display_brightness(index, brightness))
            .collect::<Vec<_>>();
        let ok = results.iter().all(|result| result.error.is_none());
        CommandResult {
            ok,
            error: None,
            results,
        }
    }

    pub fn scan(mut self, refresh: bool, command: Option<CommandResult>) -> ScanResult {
        if refresh {
            self.refresh();
        }
        let displays = self.status();
        let publications = self.collect_publications();
        ScanResult {
            controller: self,
            displays,
            publications,
            command,
        }
    }

    fn set_display_brightness(&mut self, index: usize, brightness: u8) -> BrightnessResult {
        let display = &mut self.displays[index];
        let id = display.info.id.clone();
        let name = display
            .info
            .model_name
            .clone()
            .unwrap_or_else(|| "Unknown display".to_owned());

        debug!(
            display_index = index,
            requested_percent = brightness,
            "setting brightness"
        );
        let result = (|| -> Result<u16> {
            let current = display
                .handle
                .get_vcp_feature(BRIGHTNESS)
                .context("display does not provide VCP luminance (0x10)")?;
            if current.maximum() == 0 {
                bail!("display reported a maximum luminance of zero");
            }
            let value = (u32::from(current.maximum()) * u32::from(brightness) / 100) as u16;
            display
                .handle
                .set_vcp_feature(BRIGHTNESS, value)
                .context("could not set VCP luminance (0x10)")?;
            Ok(value)
        })();

        match result {
            Ok(value) => {
                info!(
                    display_index = index,
                    requested_percent = brightness,
                    vcp_value = value,
                    "brightness updated"
                );
                BrightnessResult {
                    index,
                    id,
                    name,
                    requested_percent: brightness,
                    applied_vcp_value: Some(value),
                    error: None,
                }
            }
            Err(error) => {
                warn!(display_index = index, requested_percent = brightness, %error, "could not update brightness");
                BrightnessResult {
                    index,
                    id,
                    name,
                    requested_percent: brightness,
                    applied_vcp_value: None,
                    error: Some(error.to_string()),
                }
            }
        }
    }

    pub fn collect_publications(&mut self) -> Vec<Publication> {
        let mut publications = Vec::new();
        for (index, display) in self.displays.iter_mut().enumerate() {
            let display_topic = format!("displays/{index}");
            publications.push(publication(
                format!("{display_topic}/identity"),
                display_identity(display),
            ));
            publications.extend(display_publications(display, index, &display_topic));
        }
        publications
    }
}

fn display_publications(
    display: &mut Display,
    index: usize,
    display_topic: &str,
) -> Vec<Publication> {
    let mut publications = Vec::with_capacity(FEATURES.len() * 3);
    for feature in FEATURES {
        trace!(
            display_index = index,
            feature = feature.topic,
            vcp_code = format_args!("{:02x}", feature.code),
            "reading VCP feature"
        );
        publications.extend(read_vcp_publications(
            display,
            index,
            display_topic,
            *feature,
        ));
    }

    info!(
        display_index = index,
        vcp_count = FEATURES.len(),
        "DDC/CI display scan complete"
    );
    publications
}

fn read_vcp_publications(
    display: &mut Display,
    index: usize,
    display_topic: &str,
    feature: Feature,
) -> Vec<Publication> {
    let topic = format!("{display_topic}/{}", feature.topic);
    match display.handle.get_vcp_feature(feature.code) {
        Ok(value) => vec![
            publication(format!("{topic}/value"), value.value()),
            publication(format!("{topic}/maximum"), value.maximum()),
            publication(format!("{topic}/value_type"), value.ty),
        ],
        Err(error) => {
            // Publish null to overwrite a retained value when a monitor stops exposing a feature.
            debug!(display_index = index, feature = feature.topic, vcp_code = format_args!("{:02x}", feature.code), %error, "VCP feature is not readable");
            vec![
                publication(format!("{topic}/value"), json!(null)),
                publication(format!("{topic}/maximum"), json!(null)),
                publication(format!("{topic}/value_type"), json!(null)),
            ]
        }
    }
}

fn publication(topic_suffix: String, payload: impl Serialize) -> Publication {
    Publication {
        topic_suffix,
        payload: serde_json::to_value(payload).expect("status structures must serialize"),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug, Serialize)]
struct DisplayIdentity {
    backend: String,
    id: String,
    manufacturer_id: Option<String>,
    model_id: Option<u16>,
    version: Option<String>,
    serial: Option<u32>,
    manufacture_year: Option<u8>,
    manufacture_week: Option<u8>,
    model_name: Option<String>,
    serial_number: Option<String>,
    mccs_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    edid_hex: Option<String>,
}

fn display_identity(display: &Display) -> DisplayIdentity {
    DisplayIdentity {
        backend: display.info.backend.to_string(),
        id: display.info.id.clone(),
        manufacturer_id: display.info.manufacturer_id.clone(),
        model_id: display.info.model_id,
        version: display
            .info
            .version
            .map(|(major, minor)| format!("{major}.{minor}")),
        serial: display.info.serial,
        manufacture_year: display.info.manufacture_year,
        manufacture_week: display.info.manufacture_week,
        model_name: display.info.model_name.clone(),
        serial_number: display.info.serial_number.clone(),
        mccs_version: display.info.mccs_version.map(|version| version.to_string()),
        edid_hex: display.info.edid_data.as_deref().map(hex),
    }
}

#[cfg(test)]
mod tests {
    use super::{FEATURES, hex};

    #[test]
    fn hex_encodes_binary_edid_data() {
        assert_eq!(hex(&[0x00, 0xff, 0x10]), "00ff10");
    }

    #[test]
    fn configured_features_have_readable_unique_topic_names() {
        let topics = FEATURES
            .iter()
            .map(|feature| feature.topic)
            .collect::<Vec<_>>();
        assert_eq!(
            topics,
            [
                "brightness",
                "contrast",
                "red_gain",
                "green_gain",
                "blue_gain",
                "input_source",
                "speaker_volume",
                "power_mode"
            ]
        );
    }
}
