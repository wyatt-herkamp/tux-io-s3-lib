use std::sync::Arc;

use http::HeaderValue;
use reqwest::{Client, ClientBuilder};
use tokio::sync::RwLock;
use tracing::{debug, error};
use tux_io_s3_types::{
    credentials::Credentials,
    region::{RegionType, S3Implementation, S3Region},
};

use crate::client::{BucketClient, S3Client, S3ClientInner, settings::AccessType};
/// Runtime User Agent Value
static USER_AGENT_ENV_KEY: &'static str = "TUX_IO_S3_USER_AGENT";
/// Compile Time USER_AGENT_OVERRIDE
static USER_AGENT_DEFAULT: Option<&'static str> = option_env!("TUX_IO_DEFAULT_USER_AGENT");
/// Fall Back to `tux-io-s3/{VERSION}`
const BUILT_IN_DEFAULT_USER_AGENT: HeaderValue =
    HeaderValue::from_static(concat!("tux-io-s3/", env!("CARGO_PKG_VERSION"), "/"));
#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("Missing region")]
    MissingRegion,
    #[error(transparent)]
    HttpReqwestClientBuilderError(#[from] reqwest::Error),
}
/// First Pulls [USER_AGENT_ENV_KEY] then falls back to [USER_AGENT_DEFAULT] and finally [BUILT_IN_DEFAULT_USER_AGENT]
fn default_user_agent() -> HeaderValue {
    if let Ok(user_agent) = std::env::var(USER_AGENT_ENV_KEY) {
        match HeaderValue::from_str(&user_agent) {
            Ok(header) => return header,
            Err(err) => {
                error!(
                    ?err,
                    ?user_agent,
                    "Invalid user agent from environment variable, using default"
                );
            }
        }
    }

    if let Some(user_agent) = USER_AGENT_DEFAULT {
        match HeaderValue::from_str(user_agent) {
            Ok(header) => return header,
            Err(err) => {
                panic!("Invalid user agent from compile time constant {user_agent} err: {err}");
            }
        }
    } else {
        return BUILT_IN_DEFAULT_USER_AGENT;
    }
}
#[derive(Debug)]
pub struct S3ClientBuilder {
    region: Option<S3Region>,
    client_builder: reqwest::ClientBuilder,
    access_type: Option<AccessType>,
    credentials: Option<Credentials>,
}
impl Default for S3ClientBuilder {
    fn default() -> Self {
        Self {
            region: None,
            client_builder: Client::builder().user_agent(default_user_agent()),
            access_type: None,
            credentials: None,
        }
    }
}

impl S3ClientBuilder {
    pub fn http_client_builder<F>(mut self, builder: F) -> Self
    where
        F: FnOnce(ClientBuilder) -> ClientBuilder,
    {
        self.client_builder = builder(self.client_builder);
        self
    }
    pub fn with_http_client(mut self, client: ClientBuilder) -> Self {
        self.client_builder = client;
        self
    }
    pub fn with_access_type(mut self, access_type: AccessType) -> Self {
        self.access_type = Some(access_type);
        self
    }
    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }
    pub fn with_region(mut self, region: impl Into<S3Region>) -> Self {
        self.region = Some(region.into());
        self
    }
    fn inner_client(self) -> Result<Arc<S3ClientInner>, BuilderError> {
        let client: Client = self.client_builder.build()?;
        let credentials = if let Some(creds) = self.credentials {
            creds
        } else if let Some(credentials) = Credentials::load_from_local() {
            debug!("Loaded Local credentials");
            credentials
        } else {
            Credentials::default()
        };
        let region = self.region.ok_or(BuilderError::MissingRegion)?;

        let access_type = self
            .access_type
            .unwrap_or_else(|| match region.implementation() {
                S3Implementation::AWS => AccessType::VirtualHostedStyle,
                _ => AccessType::PathStyle,
            });

        let inner = S3ClientInner {
            http_client: client,
            region: region,
            access_type,
            credentials: RwLock::new(credentials),
        };
        Ok(Arc::new(inner))
    }
    pub fn build(self) -> Result<S3Client, BuilderError> {
        let inner = self.inner_client()?;
        Ok(S3Client { client: inner })
    }
    pub fn bucket_client(self, bucket: impl Into<String>) -> Result<BucketClient, BuilderError> {
        let inner = self.inner_client()?;
        let result = BucketClient {
            client: inner,
            bucket: bucket.into(),
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tux_io_s3_types::region::OfficialRegion;

    use super::*;
    #[test]
    fn test_s3_client_builder() {
        let builder = S3ClientBuilder::default()
            .with_region(OfficialRegion::UsEast1)
            .http_client_builder(|client| client.timeout(Duration::from_secs(30)))
            .with_access_type(AccessType::VirtualHostedStyle)
            .with_credentials(Credentials::default());

        let _client = builder.build().unwrap();
    }
}
