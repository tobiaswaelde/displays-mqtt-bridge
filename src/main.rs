mod app;
mod cli;
mod config;
mod control;
mod mqtt;
mod protocol;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::{cli::Args, config::load_config};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // Defaults to lifecycle messages; set RUST_LOG for diagnostics.
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    app::run(load_config(&args.config)?).await
}
