pub mod list;
pub mod service;

use std::borrow::Cow;

use crate::{
    AppInstance,
    commands::{list::ListCommand, service::ServiceCommand},
};
use clap::Subcommand;
use thiserror::Error;
use tux_io_s3::client::HttpResponseError;
#[derive(Debug, Clone, Subcommand)]
pub enum ClientCommand {
    Service(ServiceCommand),
    List(ListCommand),
}
#[derive(Debug, Error)]
pub enum CommandExecError {
    #[error("Failed to execute command: {0}")]
    HttpError(#[source] Box<HttpResponseError>),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}
impl From<HttpResponseError> for CommandExecError {
    fn from(error: HttpResponseError) -> Self {
        CommandExecError::HttpError(Box::new(error))
    }
}

pub(crate) trait CommandExec {
    async fn exec(self, app_instance: AppInstance) -> Result<(), CommandExecError>;
}
#[derive(Debug, Clone)]
pub struct CommandPath<'cli> {
    pub path: Cow<'cli, str>,
    pub service: Cow<'cli, str>,
    pub bucket: Cow<'cli, str>,
}
pub trait PathCommand {
    fn path(&self) -> &str;

    fn cli_service(&self) -> Option<&str>;

    fn cli_bucket(&self) -> Option<&str>;

    fn deliminator(&self) -> &str;

    fn parse_path(&self) -> Result<CommandPath<'_>, CommandExecError> {
        match (self.cli_service(), self.cli_bucket()) {
            (Some(service), Some(bucket)) => Ok(CommandPath {
                path: Cow::Borrowed(self.path()),
                service: Cow::Borrowed(service),
                bucket: Cow::Borrowed(bucket),
            }),
            (Some(service), None) => {
                let path_split: Vec<_> = self.path().split(self.deliminator()).collect();
                if path_split.len() < 2 {
                    return Err(CommandExecError::InvalidPath(format!(
                        "Path '{}' is too short to determine bucket",
                        self.path()
                    )));
                }
                let bucket: &str = path_split[0];
                let path = self.path()[(bucket.len() + self.deliminator().len())..]
                    .trim_start_matches(self.deliminator());
                Ok(CommandPath {
                    path: Cow::Borrowed(path),
                    service: Cow::Borrowed(service),
                    bucket: Cow::Borrowed(bucket),
                })
            }
            (None, Some(bucket)) => {
                let path_split: Vec<_> = self.path().split(self.deliminator()).collect();
                if path_split.is_empty() {
                    return Err(CommandExecError::InvalidPath(format!(
                        "Path '{}' is empty",
                        self.path()
                    )));
                }
                let service: &str = path_split[0];
                let path = self.path()[(service.len() + self.deliminator().len())..]
                    .trim_start_matches(self.deliminator());
                Ok(CommandPath {
                    path: Cow::Borrowed(path),
                    service: Cow::Borrowed(service),
                    bucket: Cow::Borrowed(bucket),
                })
            }
            (None, None) => {
                let path_split: Vec<_> = self.path().split(self.deliminator()).collect();
                if path_split.len() < 3 {
                    return Err(CommandExecError::InvalidPath(format!(
                        "Path '{}' is empty",
                        self.path()
                    )));
                }
                let service = path_split[0];
                let bucket = path_split[1];
                let deliminators_len = self.deliminator().len() * 2;
                let path = self.path()[(service.len() + bucket.len() + deliminators_len)..]
                    .trim_start_matches(self.deliminator());
                Ok(CommandPath {
                    path: Cow::Borrowed(path),
                    service: Cow::Borrowed(service),
                    bucket: Cow::Borrowed(bucket),
                })
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::commands::PathCommand;

    pub struct TestPathCommand {
        pub path: &'static str,
        pub service: Option<&'static str>,
        pub bucket: Option<&'static str>,
        pub deliminator: &'static str,
    }
    impl PathCommand for TestPathCommand {
        fn path(&self) -> &str {
            self.path
        }

        fn cli_service(&self) -> Option<&str> {
            self.service
        }

        fn cli_bucket(&self) -> Option<&str> {
            self.bucket
        }

        fn deliminator(&self) -> &str {
            self.deliminator
        }
    }
    #[test]
    fn simple_tests() {
        let cmd = TestPathCommand {
            path: "/test/path",
            service: Some("test-service"),
            bucket: Some("test-bucket"),
            deliminator: "/",
        };
        let parsed = cmd.parse_path();
        assert!(parsed.is_ok());
        let command_path = parsed.unwrap();
        assert_eq!(command_path.path, Cow::Borrowed("/test/path"));
        assert_eq!(command_path.service, Cow::Borrowed("test-service"));
        assert_eq!(command_path.bucket, Cow::Borrowed("test-bucket"));
    }
    #[test]
    fn service_provided() {
        let cmd = TestPathCommand {
            path: "bucket/test/path",
            service: Some("test-service"),
            bucket: None,
            deliminator: "/",
        };
        let parsed = cmd.parse_path();
        assert!(parsed.is_ok());
        let command_path = parsed.unwrap();
        println!("{:?}", command_path);
        assert_eq!(command_path.path, Cow::Borrowed("test/path"));
        assert_eq!(command_path.service, Cow::Borrowed("test-service"));
        assert_eq!(command_path.bucket, Cow::Borrowed("bucket"));
    }
    #[test]
    fn bucket_provided() {
        let cmd = TestPathCommand {
            path: "service/test/path",
            service: None,
            bucket: Some("test-bucket"),
            deliminator: "/",
        };
        let parsed = cmd.parse_path();
        assert!(parsed.is_ok());
        let command_path = parsed.unwrap();
        println!("{:?}", command_path);
        assert_eq!(command_path.path, Cow::Borrowed("test/path"));
        assert_eq!(command_path.service, Cow::Borrowed("service"));
        assert_eq!(command_path.bucket, Cow::Borrowed("test-bucket"));
    }
    #[test]
    fn non_provided_bucket() {
        let cmd = TestPathCommand {
            path: "service/bucket/test/path",
            service: None,
            bucket: None,
            deliminator: "/",
        };
        let parsed = cmd.parse_path();
        assert!(parsed.is_ok());
        let command_path = parsed.unwrap();
        println!("{:?}", command_path);
        assert_eq!(command_path.path, Cow::Borrowed("test/path"));
        assert_eq!(command_path.service, Cow::Borrowed("service"));
        assert_eq!(command_path.bucket, Cow::Borrowed("bucket"));
    }
}
