use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Command {
    SetBrightness {
        display: DisplayTarget,
        brightness: u8,
    },
    ListDisplays,
    ScanDisplays,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DisplayTarget {
    All(String),
    Index(usize),
}

impl DisplayTarget {
    pub fn indices(&self, count: usize) -> Result<Vec<usize>> {
        match self {
            Self::All(value) if value == "all" => Ok((0..count).collect()),
            Self::All(value) => {
                bail!("display must be \"all\" or a zero-based numeric index, got {value:?}")
            }
            Self::Index(index) if *index < count => Ok(vec![*index]),
            Self::Index(index) => {
                bail!("display index {index} does not exist; detected displays: {count}")
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Status {
    pub displays: Vec<DisplayStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<CommandResult>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DisplayStatus {
    pub index: usize,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CommandResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<BrightnessResult>,
}

impl CommandResult {
    pub fn success() -> Self {
        Self {
            ok: true,
            error: None,
            results: vec![],
        }
    }

    pub fn failure(error: impl ToString) -> Self {
        Self {
            ok: false,
            error: Some(error.to_string()),
            results: vec![],
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BrightnessResult {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub requested_percent: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_vcp_value: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct Publication {
    pub topic_suffix: String,
    pub payload: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::{Command, DisplayTarget};

    #[test]
    fn all_target_selects_every_display() {
        assert_eq!(
            DisplayTarget::All("all".to_owned()).indices(3).unwrap(),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn index_target_rejects_unknown_display() {
        assert!(DisplayTarget::Index(2).indices(2).is_err());
    }

    #[test]
    fn set_brightness_command_accepts_all_displays() {
        let command: Command =
            serde_json::from_str(r#"{"command":"set_brightness","display":"all","brightness":60}"#)
                .unwrap();

        assert!(matches!(
            command,
            Command::SetBrightness { brightness: 60, .. }
        ));
    }

    #[test]
    fn scan_displays_command_is_supported() {
        assert!(matches!(
            serde_json::from_str::<Command>(r#"{"command":"scan_displays"}"#).unwrap(),
            Command::ScanDisplays
        ));
    }
}
