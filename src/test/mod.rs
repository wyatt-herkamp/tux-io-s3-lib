use std::{path::PathBuf, sync::{Arc, Once}};

use serde::{Deserialize, Serialize};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tux_io_s3_types::{credentials::Credentials, region::S3Region};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestConfig {
    pub region: S3Region,
    pub credentials: Credentials,
    pub bucket: String,
}

pub fn init_test_logger() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        if std::env::var("RUST_LOG").is_err() {
            // If no RUST_LOG is set a default for current instance
            unsafe {
                std::env::set_var("RUST_LOG", "info,tux_io_s3=trace");
            }
        }
        let writer = tracing_subscriber::fmt::layer()
            .without_time()
            .with_thread_ids(false)
            .with_thread_names(false)
            .pretty()
            .with_writer(std::io::stderr);
        let _ = tracing_subscriber::registry()
            .with(writer)
            .with(EnvFilter::from_default_env())
            .try_init();
    });
}
fn test_config_path() -> PathBuf {
    if let Ok(path) = std::env::var("TEST_CONFIG_PATH") {
        PathBuf::from(path)
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test.toml")
    }
}
pub fn load_testing_region_and_credentials() -> (S3Region, Credentials, String) {
    let config_path = test_config_path();
    if !config_path.exists() {
        panic!("Test configuration file not found at: {:?}", config_path);
    }
    let config: TestConfig = match std::fs::read_to_string(&config_path) {
        Ok(content) => toml::from_str(&content).expect("Failed to parse test configuration"),
        Err(e) => panic!("Failed to read test configuration file: {}", e),
    };
    (config.region, config.credentials, config.bucket)
}
pub fn create_test_bucket_client() -> crate::client::BucketClient {
    let (region, credentials, bucket) = load_testing_region_and_credentials();
    crate::client::S3ClientBuilder::default()
        .with_region(region)
        .with_access_type(crate::client::AccessType::PathStyle)
        .with_credentials(Arc::new(credentials.into()))
        .bucket_client(&bucket)
        .expect("Failed to create test bucket client")
}
pub fn create_test_client() -> crate::client::S3Client {
    let (region, credentials, _) = load_testing_region_and_credentials();
    crate::client::S3ClientBuilder::default()
        .with_region(region)
        .with_access_type(crate::client::AccessType::PathStyle)
        .with_credentials(Arc::new(credentials.into()))
        .build()
        .expect("Failed to create test bucket client")
}

mod tests {
    use crate::test::init_test_logger;

    #[test]
    fn test_init_logger() {
        init_test_logger();
        tracing::info!("Logger initialized for tests");
    }
}
