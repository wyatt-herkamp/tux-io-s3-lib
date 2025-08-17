use std::sync::Arc;

use http::{HeaderValue, StatusCode};
use reqwest::Response;
use tracing::{debug, field::Empty};
use tux_io_s3_types::{S3ContentError, list::v2, region::RegionType, tag::OwnedTaggingSet};
use url::Url;

use crate::{
    S3Error,
    client::{S3ClientInner, errors::HttpResponseError, settings::AccessType},
    command::{
        BucketCommandType, CommandType,
        get::{GetObject, GetObjectResponse, GetObjectTagging},
        head::{HeadObject, HeadObjectResponse},
        list::ListObjectsV2,
    },
};
#[derive(Debug, Clone)]
pub struct BucketClient {
    pub(crate) client: Arc<S3ClientInner>,
    pub(crate) bucket: String,
}
impl BucketClient {
    pub fn url(&self) -> Result<Url, S3Error> {
        match self.client.access_type {
            AccessType::PathStyle => {
                let mut url = self.client.region.endpoint_url();
                url.set_path(&format!("/{}/", self.bucket));
                Ok(url)
            }
            AccessType::VirtualHostedStyle => {
                let raw_host = self.client.region.endpoint();
                let url = format!(
                    "{}://{}.{}",
                    self.client.region.schema(),
                    self.bucket,
                    raw_host
                );
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
                Ok(format!("{}.{}", self.bucket, raw_host))
            }
        }
    }
    /// Calls the HEAD operation on the object at the given path.
    ///
    /// For the sake of consistency, the return is an `Option<Response>`. Ok(None) means the object does not exist.
    /// Any other HTTP Error will be returned as [S3Error::HttpError].
    pub async fn head_object(&self, path: &str) -> Result<Option<HeadObjectResponse>, S3Error> {
        let command = HeadObject { key: path };
        let result = self.execute_command(command).await?;
        if result.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !result.status().is_success() {
            return Err(HttpResponseError::from(result).into());
        }
        Ok(Some(HeadObjectResponse(result)))
    }
    /// Returns the tagging set for the object at the given path.
    ///
    /// If the object does not exist, returns `None`.
    /// Any other HTTP Error will be returned as [S3Error::HttpError].
    pub async fn get_object_tagging(&self, key: &str) -> Result<Option<OwnedTaggingSet>, S3Error> {
        let command = GetObjectTagging {
            key,
            ..Default::default()
        };
        let result = self.execute_command(command).await?;
        if result.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !result.status().is_success() {
            return Err(HttpResponseError::from(result).into());
        }
        let body = result.text().await?;

        let tagging_set: OwnedTaggingSet =
            quick_xml::de::from_str(&body).map_err(S3ContentError::from)?;
        Ok(Some(tagging_set))
    }
    pub async fn list_objects_v2(
        &self,
        list_objects: impl Into<ListObjectsV2<'_>>,
    ) -> Result<v2::ListBucketResult, S3Error> {
        let command = list_objects.into();
        let response = self.execute_command(command).await?;
        if !response.status().is_success() {
            return Err(HttpResponseError::from(response).into());
        }
        let body = response.text().await?;
        debug!("ListObjects response body: {}", body);
        let data: v2::ListBucketResult =
            quick_xml::de::from_str(&body).map_err(S3ContentError::from)?;
        Ok(data)
    }
    pub async fn get_object(&self, key: &str) -> Result<Option<GetObjectResponse>, S3Error> {
        let command = GetObject {
            key,
            ..Default::default()
        };
        let response = self.execute_command(command).await?;
        if !response.status().is_success() {
            return Err(HttpResponseError::from(response).into());
        }
        Ok(Some(GetObjectResponse(response.into())))
    }
    /// A low-level method to execute any [BucketCommandType].
    ///
    /// This will return Ok(Response) for any HTTP Response that could be parsed by the HTTP Client.
    ///
    /// This handles all the nessary authentication and request preparation.
    ///
    /// All of the other functions for this Client use this method internally.
    pub async fn execute_command<'request, T>(
        &'request self,
        command: T,
    ) -> Result<Response, S3Error>
    where
        T: CommandType + BucketCommandType + Send + 'request,
    {
        let span = tracing::debug_span!(
            "S3 Command",
            command = command.name(),
            bucket = self.bucket,
            method = command.http_method().as_str(),
            status_code = Empty
        );
        let _enter = span.enter();
        let _enter = span.enter();
        let url = self.url()?;
        debug!(%url, "Executing S3 command");
        let host = HeaderValue::from_str(&self.host()?)?;

        let response = self.client.execute_command(command, url, host).await?;

        span.record("status_code", response.status().as_u16());
        debug!("S3 Command Responded");

        Ok(response)
    }
}
