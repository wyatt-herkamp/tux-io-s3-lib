use std::sync::Arc;

use http::HeaderValue;
use reqwest::Client;
use tokio::sync::RwLock;
use tracing::debug;
use tux_io_s3_types::{
    credentials::Credentials,
    region::{RegionType, S3Implementation, S3Region},
};

use crate::client::{
    BucketClient, S3Client, S3ClientInner,
    settings::{AccessType, ListObjectsVersion},
};

#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("Missing region")]
    MissingRegion,
    #[error(transparent)]
    HttpReqwestClientBuilderError(#[from] reqwest::Error),
}
fn default_user_agent() -> HeaderValue {
    HeaderValue::from_static(concat!("tux-io-s3/", env!("CARGO_PKG_VERSION")))
}
#[derive(Debug, Clone, Default)]
pub struct S3ClientBuilder {
    region: Option<S3Region>,
    client: Option<Client>,
    access_type: Option<AccessType>,
    credentials: Option<Credentials>,
}

impl S3ClientBuilder {
    pub fn with_http_client(mut self, client: Client) -> Self {
        self.client = Some(client);
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
    pub fn with_region(mut self, region: S3Region) -> Self {
        self.region = Some(region);
        self
    }
    fn inner_client(self) -> Result<Arc<S3ClientInner>, BuilderError> {
        let client: Client = match self.client {
            Some(c) => c,
            None => Client::builder()
                .user_agent(default_user_agent())
                .build()
                .map_err(BuilderError::HttpReqwestClientBuilderError)?,
        };
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
