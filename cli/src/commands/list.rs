use clap::Parser;

use crate::{
    AppInstance,
    commands::{CommandExec, CommandExecError},
};
#[derive(Debug, Clone, Parser)]
pub struct ListCommand {
    #[arg(long, default_value = "/")]
    pub deliminator: String,
    /// Which service to use
    ///
    /// If not set it will use the first part of the path
    #[arg(long)]
    pub service: Option<String>,
    /// Which bucket to use
    ///
    /// If not set it will use the second part of the path
    #[arg(long)]
    pub bucket: Option<String>,
    /// The path to list
    pub path: String,
}
impl CommandExec for ListCommand {
    async fn exec(self, app_instance: AppInstance) -> Result<(), CommandExecError> {
        Ok(())
    }
}
