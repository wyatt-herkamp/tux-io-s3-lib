use std::{borrow::Cow, sync::Arc};

use http::{
    HeaderValue,
    header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_LENGTH, DATE, HOST, TRANSFER_ENCODING},
};
use reqwest::{Body, Response};
use tracing::{debug, error};
use tux_io_s3_types::{
    Service,
    headers::{X_AMZ_CONTENT_SHA256, X_AMZ_DATE, X_AMZ_DECODED_CONTENT_LENGTH},
    region::{RegionType, S3Region},
};
use url::Url;

use crate::{
    EMPTY_HASH, S3Error,
    client::{http_client::HttpClient, settings::AccessType},
    command::{
        CommandType,
        body::{FixedStream, S3ContentStream},
    },
    credentials::{
        header::AWS4HMACSHA256HeaderBuilder,
        provider::{CredentialsProvider, CredentialsProviderType},
        sha256_from_bytes,
    },
    utils::LONG_DATE_FORMAT,
};
#[derive(Debug)]
pub(crate) struct S3ClientInner<Client: HttpClient = reqwest::Client> {
    pub(crate) http_client: Client,
    pub(crate) region: S3Region,
    /// Should always be true for custom s3 clients.
    pub(crate) access_type: AccessType,
    pub(crate) credentials: Arc<CredentialsProvider>,
}
impl<Client: HttpClient> S3ClientInner<Client> {
    /// Internal method to execute S3 commands.
    pub(crate) async fn execute_command<'request, T>(
        &'request self,
        command: T,
        mut url: Url,
        host_name: HeaderValue,
    ) -> Result<Response, S3Error>
    where
        T: CommandType + Send + 'request,
    {
        let credentials = match self.credentials.provide(self.http_client.clone()).await {
            Ok(ok) => ok,
            Err(err) => {
                error!(
                    %err,
                    provider = %self.credentials.name(),
                    "Failed to get credentials from provider"
                );
                return Err(S3Error::CredentialsError(err));
            }
        };
        command.update_url(&mut url)?;
        debug!(%url, "Executing S3 command");
        let now = chrono::Utc::now();
        let mut headers = http::HeaderMap::new();
        headers.insert(HOST, host_name);
        headers.append(
            X_AMZ_DATE,
            HeaderValue::from_str(&now.format(LONG_DATE_FORMAT).to_string())?,
        );
        command.headers(&mut headers)?;
        let http_method = command.http_method();
        let body = command.into_body()?;
        let mut auth_header = AWS4HMACSHA256HeaderBuilder::default()
            .date_time(now)
            .region(&self.region)
            .url(&url)
            .authentication(&credentials.access_key, &credentials.secret_key);

        let fixed_body = body.inner.into_fixed_stream().await?;

        let body = match fixed_body {
            FixedStream::None => {
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
            FixedStream::FixedContent(body) => {
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
            FixedStream::Stream {
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
                    stream,
                    time: now,
                    previous_signature,
                    region: self.region.name().to_string(),
                    service: Service::S3,
                    signing_key,
                    sent_final_chunk: false,
                };

                Some(Body::wrap_stream(body_wrapper))
            }
        };
        #[cfg(test)]
        {
            tracing::info!(?headers, "Executing S3 command with headers");
        }
        let mut response = self.http_client.request(http_method, url).headers(headers);
        if let Some(body) = body {
            response = response.body(body);
        }
        let response = response.send().await?;

        Ok(response)
    }
}
