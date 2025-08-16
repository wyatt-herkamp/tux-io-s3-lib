use clap::{Parser, Subcommand};
#[derive(Debug, Clone, Parser)]
pub struct ServiceCommand {
    #[command(subcommand)]
    pub command: ServiceSubCommand,
}
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ServiceSubCommand {
    List,
}
