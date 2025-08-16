use crate::{commands::ClientCommand, config::Config};
use clap::Parser;
use std::path::PathBuf;

pub mod commands;
pub mod config;
pub mod error;
#[derive(Debug, Clone, Parser)]
pub struct CLI {
    #[command(subcommand)]
    command: ClientCommand,
}
fn main() -> anyhow::Result<()> {
    let (config, home_dir) = config::load_config()?;
    let app_instance = AppInstance { config, home_dir };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(actual_main(app_instance))
}

async fn actual_main(app_instance: AppInstance) -> anyhow::Result<()> {
    Ok(())
}

pub struct AppInstance {
    pub config: Config,
    pub home_dir: PathBuf,
}
