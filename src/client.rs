use std::sync::Arc;

mod bucket;
mod builder;
mod settings;
pub use bucket::BucketClient;
pub use builder::{BuilderError, S3ClientBuilder};
use http::HeaderValue;
use reqwest::Response;
use tracing::{debug, field::Empty};
use tux_io_s3_types::{
    list::buckets::ListAllMyBuckets,
    region::{RegionType, S3Region},
};
pub mod http_client;
use url::Url;
pub mod inner;
use crate::{
    S3Error,
    client::inner::S3ClientInner,
    command::{AccountCommandType, CommandType, list::buckets::ListBuckets},
};
mod errors;
pub use errors::*;
pub use settings::*;
pub struct S3Client {
    pub(crate) client: Arc<S3ClientInner>,
}
impl S3Client {
    pub fn get_region(&self) -> &S3Region {
        &self.client.region
    }
    /// Requires
    ///   - `s3:ListAllMyBuckets` permission
    pub async fn list_my_buckets(&self) -> Result<ListAllMyBuckets, S3Error> {
        let command = ListBuckets {
            region: &self.client.region,
            continuation_token: None,
            max_buckets: None,
            prefix: None,
        };
        let response = self.execute_command(command).await?;
        if !response.status().is_success() {
            return Err(HttpResponseError::from(response).into());
        }
        let list_buckets: ListAllMyBuckets =
            quick_xml::de::from_str(&response.text().await?).unwrap();
        Ok(list_buckets)
    }

    pub fn open_bucket(&self, bucket: &str) -> BucketClient {
        BucketClient {
            client: Arc::clone(&self.client),
            bucket: bucket.to_string(),
        }
    }

    pub async fn execute_command<'request, T>(
        &'request self,
        command: T,
    ) -> Result<Response, S3Error>
    where
        T: CommandType + AccountCommandType + Send + 'request,
    {
        let span = tracing::debug_span!(
            "S3 Command",
            command = command.name(),
            method = command.http_method().as_str(),
            status_code = Empty
        );
        let _enter = span.enter();
        let url = self.url()?;
        debug!(%url, "Executing S3 command");
        let host = HeaderValue::from_str(&self.host()?)?;

        let response = self.client.execute_command(command, url, host).await?;

        span.record("status_code", response.status().as_u16());
        debug!("S3 Command Responded");

        Ok(response)
    }

    pub fn url(&self) -> Result<Url, S3Error> {
        match self.client.access_type {
            AccessType::PathStyle => {
                let url = self.client.region.endpoint_url();

                Ok(url)
            }
            AccessType::VirtualHostedStyle => {
                let raw_host = self.client.region.endpoint();
                let url = format!("{}://{}", self.client.region.schema(), raw_host);
                Url::parse(&url).map_err(S3Error::from)
            }
        }
    }
    pub fn host(&self) -> Result<String, S3Error> {
        match self.client.access_type {
            AccessType::PathStyle => {
                if let Some(port) = self.client.region.endpoint_url().port() {
                    Ok(format!("{}:{}", self.client.region.endpoint(), port))
                } else {
                    Ok(self.client.region.endpoint().to_string())
                }
            }
            AccessType::VirtualHostedStyle => {
                let raw_host = self.client.region.endpoint();
                Ok(raw_host.to_string())
            }
        }
    }
}
