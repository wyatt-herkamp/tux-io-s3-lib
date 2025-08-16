use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tux_io_s3_types::{credentials::Credentials, region::S3Region};
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Service with name '{0}' already exists")]
    ServiceAlreadyExists(String),
    #[error("Service with name '{0}' does not exist")]
    ServiceDoesNotExist(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Home directory not found")]
    HomeDirNotFound,
    #[error(transparent)]
    TomlParse(#[from] toml::de::Error),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub region: S3Region,
    pub credentials: Credentials,
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub services: Vec<Service>,
}
impl Config {
    pub fn add_service(&mut self, service: Service) -> Result<(), ConfigError> {
        for existing_service in &self.services {
            if existing_service.name == service.name {
                return Err(ConfigError::ServiceAlreadyExists(service.name));
            }
        }
        self.services.push(service);
        Ok(())
    }
    pub fn update_service(&mut self, service: Service) -> Result<(), ConfigError> {
        for existing_service in &mut self.services {
            if existing_service.name == service.name {
                *existing_service = service;
                return Ok(());
            }
        }
        Err(ConfigError::ServiceDoesNotExist(service.name))
    }
}
pub fn load_config() -> Result<(Config, PathBuf), ConfigError> {
    let home_dir = home_dir()?;
    let config_path = home_dir.join("config.toml");
    let config_content = std::fs::read_to_string(config_path)?;
    let config = toml::from_str(&config_content)?;
    Ok((config, home_dir))
}

fn home_dir() -> Result<PathBuf, ConfigError> {
    let home_dir = if let Some(dir) = std::env::var_os("TUX_IO_S3_HOME") {
        PathBuf::from(dir)
    } else {
        std::env::home_dir().ok_or(ConfigError::HomeDirNotFound)?
    };
    let tux_io_s3_dir = home_dir.join(".tux-io-s3");
    if !tux_io_s3_dir.exists() {
        std::fs::create_dir_all(&tux_io_s3_dir)?;
    }
    Ok(tux_io_s3_dir)
}
