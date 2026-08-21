use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Control DDC/CI monitor brightness through MQTT")]
pub struct Args {
    #[arg(short, long, default_value = "/app/config/config.yml")]
    pub config: PathBuf,
}
