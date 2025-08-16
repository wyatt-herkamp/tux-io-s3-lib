use std::{borrow::Cow, sync::Arc};

use http::{
    HeaderValue, StatusCode,
    header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_LENGTH, DATE, HOST, TRANSFER_ENCODING},
};
use reqwest::{Body, Response};
use tracing::{debug, field::Empty};
use tux_io_s3_types::{
    ContentParseError, Service,
    headers::{X_AMZ_CONTENT_SHA256, X_AMZ_DATE, X_AMZ_DECODED_CONTENT_LENGTH},
    list::v2,
    region::RegionType,
    tag::OwnedTaggingSet,
};
use url::Url;

use crate::{
    EMPTY_HASH, S3Error,
    client::{S3ClientInner, errors::HttpResponseError, settings::AccessType},
    command::{
        BucketCommandType, CommandType, S3CommandBody,
        body::{S3CommandBodyInner, S3ContentStream},
        get::{GetObject, GetObjectResponse, GetObjectTagging},
        head::{HeadObject, HeadObjectResponse},
        list::ListObjectsV2,
    },
    credentials::{header::AWS4HMACSHA256HeaderBuilder, sha256_from_bytes},
    utils::LONG_DATE_FORMAT,
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
            quick_xml::de::from_str(&body).map_err(ContentParseError::from)?;
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
            quick_xml::de::from_str(&body).map_err(ContentParseError::from)?;
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
        let credentials = self.client.credentials.read().await;
        let mut url = self.url()?;
        command.update_url(&mut url)?;
        debug!(%url, "Executing S3 command");
        let now = chrono::Utc::now();
        let mut headers = http::HeaderMap::new();
        headers.insert(HOST, HeaderValue::from_str(&self.host()?)?);
        headers.append(
            X_AMZ_DATE,
            HeaderValue::from_str(&now.format(LONG_DATE_FORMAT).to_string())?,
        );
        command.headers(&mut headers)?;
        let http_method = command.http_method();
        let body = command.into_body()?;
        let mut auth_header = AWS4HMACSHA256HeaderBuilder::default()
            .date_time(now)
            .region(&self.client.region)
            .url(&url);
        {
            if let Some((access_key, secret_key)) = credentials.access_key_and_secret() {
                auth_header = auth_header.authentication(access_key, secret_key)
            }
        }

        let body = match body.inner {
            S3CommandBodyInner::None => {
                headers.append(X_AMZ_CONTENT_SHA256, HeaderValue::from_str(EMPTY_HASH)?);
                headers.append(CONTENT_LENGTH, HeaderValue::from_str(&format!("{}", 0))?);
                auth_header =
                    auth_header.request_info(http_method.clone(), Cow::Borrowed(EMPTY_HASH));
                auth_header = auth_header.headers(&headers);
                let auth_header = auth_header.build()?;
                headers.append(AUTHORIZATION, auth_header.header_value()?);
                headers.insert(DATE, HeaderValue::from_str(&now.to_rfc2822())?);
                None
            }
            S3CommandBodyInner::FixedSize(body) => {
                let sha256 = sha256_from_bytes(&body);
                let content_length = body.len();
                headers.append(X_AMZ_CONTENT_SHA256, HeaderValue::from_str(&sha256)?);
                headers.append(
                    CONTENT_LENGTH,
                    HeaderValue::from_str(&format!("{}", content_length))?,
                );
                auth_header = auth_header.request_info(http_method.clone(), Cow::Owned(sha256));
                auth_header = auth_header.headers(&headers);
                let auth_header = auth_header.build()?;
                headers.append(AUTHORIZATION, auth_header.header_value()?);
                headers.insert(DATE, HeaderValue::from_str(&now.to_rfc2822())?);
                Some(reqwest::Body::from(body))
            }
            S3CommandBodyInner::Stream {
                stream,
                content_length,
            } => {
                headers.append(
                    X_AMZ_CONTENT_SHA256,
                    HeaderValue::from_static("STREAMING-AWS4-HMAC-SHA256-PAYLOAD"),
                );
                headers.append(TRANSFER_ENCODING, HeaderValue::from_static("chunked"));

                headers.append(
                    X_AMZ_DECODED_CONTENT_LENGTH,
                    HeaderValue::from_str(&content_length.to_string())?,
                );

                headers.append(CONTENT_ENCODING, HeaderValue::from_static("aws-chunked"));

                auth_header = auth_header
                    .request_info(
                        http_method.clone(),
                        Cow::Borrowed("STREAMING-AWS4-HMAC-SHA256-PAYLOAD"),
                    )
                    .headers(&headers);

                let signing_key = auth_header.signature.key()?;

                let auth_header = auth_header.build()?;
                let previous_signature = auth_header.canonical_request.encode(&signing_key)?;
                headers.append(AUTHORIZATION, auth_header.header_value()?);
                headers.insert(DATE, HeaderValue::from_str(&now.to_rfc2822())?);
                let body_wrapper = S3ContentStream::<Box<dyn std::error::Error + Send + Sync>, _> {
                    stream: stream,
                    time: now,
                    previous_signature: previous_signature,
                    region: self.client.region.name().to_string(),
                    service: Service::S3,
                    signing_key,
                    sent_final_chunk: false,
                };

                Some(Body::wrap_stream(body_wrapper))
            }
        };
        // Only Enabled for tests
        // Just for testing purposes otherwise it would be a security risk
        #[cfg(test)]
        {
            tracing::info!(?headers, "Executing S3 command with headers");
        }
        let mut response = self
            .client
            .http_client
            .request(http_method, url)
            .headers(headers);
        if let Some(body) = body {
            response = response.body(body);
        }
        let response = response.send().await?;

        span.record("status_code", response.status().as_u16());
        debug!("S3 Command Responded");

        Ok(response)
    }
}
