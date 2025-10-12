use std::path::PathBuf;

use chrono::Utc;
use tokio::sync::RwLock;
use tracing::*;
use tux_io_s3_types::{
    credentials::{
        AssumeRoleWithWebIdentityRequest, AssumeRoleWithWebIdentityResponse, Credentials,
        StsResponseCredentials,
    },
    utils::DurationSeconds,
};
use url::Url;

use crate::{
    client::{HttpResponseError, http_client::HttpClient},
    credentials::provider::{CredentialsProviderError, CredentialsProviderType},
};
#[derive(Debug, thiserror::Error)]
pub enum AssumeRoleWithWebIdentityError {
    #[error("Web Identity Token File does not exist {0}")]
    TokenFileDoesNotExist(PathBuf),
    #[error("Failed to Read Token File {0}")]
    FailedToReadTokenFile(#[from] std::io::Error),
    #[error(transparent)]
    HttpError(Box<HttpResponseError>),
    #[error("Failed to parse STS Response {0}")]
    FailedToParseStsResponse(#[from] quick_xml::de::DeError),
}
impl AssumeRoleWithWebIdentityError {
    pub fn status_code(&self) -> Option<http::StatusCode> {
        match self {
            AssumeRoleWithWebIdentityError::HttpError(err) => err.status_code(),
            _ => None,
        }
    }
}
impl From<HttpResponseError> for AssumeRoleWithWebIdentityError {
    fn from(error: HttpResponseError) -> Self {
        AssumeRoleWithWebIdentityError::HttpError(Box::new(error))
    }
}
impl From<reqwest::Error> for AssumeRoleWithWebIdentityError {
    fn from(error: reqwest::Error) -> Self {
        let response = HttpResponseError::from(error);
        Self::from(response)
    }
}
/// AWS STS AssumeRoleWithWebIdentity Credentials Provider
#[derive(Debug)]
pub struct AssumeRoleWithWebIdentity {
    pub role_arn: String,
    pub web_identity_token: PathBuf,
    pub sts_endpoint: Url,
    pub session_name: String,
    pub timeout: Option<std::time::Duration>,
    pub token_duration: Option<DurationSeconds>,
    /// Wrapped in a Box to reduce the size of the struct
    cached_credentials: Box<RwLock<Option<StsResponseCredentials>>>,
}
impl Clone for AssumeRoleWithWebIdentity {
    fn clone(&self) -> Self {
        let cached_credentials = if let Ok(current) = tokio::runtime::Handle::try_current() {
            current.block_on(async { self.cached_credentials.read().await.clone() })
        } else {
            self.cached_credentials.blocking_read().clone()
        };
        Self {
            role_arn: self.role_arn.clone(),
            web_identity_token: self.web_identity_token.clone(),
            sts_endpoint: self.sts_endpoint.clone(),
            session_name: self.session_name.clone(),
            timeout: self.timeout,
            token_duration: self.token_duration,
            cached_credentials: Box::new(RwLock::new(cached_credentials)),
        }
    }
}
impl PartialEq for AssumeRoleWithWebIdentity {
    fn eq(&self, other: &Self) -> bool {
        let result = self.role_arn == other.role_arn
            && self.web_identity_token == other.web_identity_token
            && self.sts_endpoint == other.sts_endpoint
            && self.session_name == other.session_name
            && self.timeout == other.timeout;
        // If the basic fields are not equal, return false immediately
        if !result {
            return false;
        }
        // Attempts to get the current tokio runtime to do an async read lock
        // If that fails, we do a blocking read lock
        let cached_credentials = if let Ok(current) = tokio::runtime::Handle::try_current() {
            current.block_on(async {
                let self_creds = self.cached_credentials.read().await;
                let other_creds = other.cached_credentials.read().await;
                *self_creds == *other_creds
            })
        } else {
            let self_creds = self.cached_credentials.blocking_read();
            let other_creds = other.cached_credentials.blocking_read();
            *self_creds == *other_creds
        };
        result && cached_credentials
    }
}
impl Eq for AssumeRoleWithWebIdentity {}
impl AssumeRoleWithWebIdentity {
    /// Creates a new AssumeRoleWithWebIdentity provider.
    pub fn new(
        role_arn: String,
        web_identity_token: PathBuf,
        sts_endpoint: Url,
        session_name: String,
    ) -> Self {
        Self {
            role_arn,
            web_identity_token,
            sts_endpoint,
            session_name,
            timeout: None,
            token_duration: None,
            cached_credentials: Box::new(RwLock::new(None)),
        }
    }
    /// Sets the request timeout for the STS AssumeRoleWithWebIdentity request.
    /// If not set, the default reqwest timeout will be used.
    pub fn with_request_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    /// Sets the duration for the assumed role session. If not set, the default
    /// duration set by AWS STS will be used (usually 1 hour).
    pub fn with_token_duration(mut self, duration: DurationSeconds) -> Self {
        self.token_duration = Some(duration);
        self
    }
    /// Attempts to get cached credentials if they exist and are still valid.
    async fn get_cached_credentials(&self) -> Option<Credentials> {
        {
            let creds = self.cached_credentials.read().await;
            creds
                .as_ref()
                .filter(|creds| creds.expiration > Utc::now())
                .map(Credentials::from)
        }
    }
    /// Returns true if the current cached credentials exist and are still valid.
    /// Otherwise, returns false.
    pub async fn is_valid(&self) -> bool {
        self.get_cached_credentials().await.is_some()
    }
    /// Forces a refresh of the credentials, ignoring any cached credentials.
    pub async fn force_refresh(
        &self,
        client: &impl HttpClient,
    ) -> Result<Credentials, AssumeRoleWithWebIdentityError> {
        let new_creds = self.request_credentials(client).await?;

        let result_creds = Credentials::from(&new_creds);
        {
            let mut write_lock = self.cached_credentials.write().await;
            *write_lock = Some(new_creds);
        }
        Ok(result_creds)
    }
    async fn read_token_file(&self) -> Result<String, AssumeRoleWithWebIdentityError> {
        if !self.web_identity_token.exists() {
            return Err(AssumeRoleWithWebIdentityError::TokenFileDoesNotExist(
                self.web_identity_token.clone(),
            ));
        }
        let token = tokio::fs::read_to_string(&self.web_identity_token).await?;
        Ok(token)
    }
    async fn request_credentials(
        &self,
        client: &impl HttpClient,
    ) -> Result<StsResponseCredentials, AssumeRoleWithWebIdentityError> {
        let token = self.read_token_file().await?;
        let request_query = AssumeRoleWithWebIdentityRequest {
            role_arn: self.role_arn.clone(),
            web_identity_token: token,
            role_session_name: self.session_name.clone(),
            duration_seconds: self.token_duration,
            ..Default::default()
        };
        #[cfg(test)]
        {
            info!("AssumeRoleWithWebIdentity Request: {:#?}", request_query);
        }
        let mut request = client.get(self.sts_endpoint.clone()).query(&request_query);
        if let Some(timeout) = self.timeout {
            request = request.timeout(timeout);
        }
        let request = request.build()?;

        let response = client.execute(request).await?;
        if !response.status().is_success() {
            error!(
                status_code = %response.status(),
                "AssumeRoleWithWebIdentity Request failed"
            );
            return Err(AssumeRoleWithWebIdentityError::HttpError(Box::new(
                HttpResponseError::from(response),
            )));
        } else {
            debug!("AssumeRoleWithWebIdentity Request Succeeded");
        }

        let sts_response: String = response.text().await?;
        let sts_response: AssumeRoleWithWebIdentityResponse =
            quick_xml::de::from_str(&sts_response)?;

        debug!(
            request_id = %sts_response.response_metadata.request_id,
            expiration = %sts_response.assume_role_with_web_identity_result.credentials.expiration,
            "Successfully assumed role with web identity"
        );

        Ok(sts_response
            .assume_role_with_web_identity_result
            .credentials)
    }
}
impl CredentialsProviderType for AssumeRoleWithWebIdentity {
    fn name(&self) -> &'static str {
        "AssumeRoleWithWebIdentityProvider"
    }
    #[instrument(
        level = "debug",
        skip(self, client)
        fields(role_arn = %self.role_arn, sts_endpoint = %self.sts_endpoint, session_name = %self.session_name, reloaded)
    )]
    async fn provide(
        &self,
        client: impl HttpClient,
    ) -> Result<Credentials, CredentialsProviderError> {
        let cached = self.get_cached_credentials().await;
        if let Some(creds) = cached {
            Span::current().record("reloaded", false);
            return Ok(creds);
        }
        Span::current().record("reloaded", true);
        let mut write_lock = self.cached_credentials.write().await;
        if let Some(creds) = write_lock.as_ref() {
            // Double check the creds after acquiring the write lock
            if creds.expiration > Utc::now() {
                return Ok(Credentials::from(creds));
            }
        }
        // Set it to none Just in case we error out while fetching new creds
        *write_lock = None;

        let new_creds = self.request_credentials(&client).await?;

        let result_creds = Credentials::from(&new_creds);
        *write_lock = Some(new_creds);
        drop(write_lock);
        Ok(result_creds)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use chrono::{Duration, Local};
    use http::{HeaderMap, StatusCode};
    use tux_io_s3_types::credentials::{
        AssumeRoleWithWebIdentityResponse, AssumeRoleWithWebIdentityResult, AssumedRoleUser, ResponseMetadata, StsResponseCredentials
    };

    use crate::{
        client::http_client::{MockOkClient, MockResponse},
        credentials::provider::CredentialsProviderType,
    };

    #[test]
    fn compare_tests() {
        let provider1 = super::AssumeRoleWithWebIdentity::new(
            "arn:aws:iam::123456789012:role/WebIdentityRole".to_string(),
            "/path/to/token".into(),
            "https://sts.amazonaws.com".parse().unwrap(),
            "session-name".to_string(),
        );
        let provider2 = super::AssumeRoleWithWebIdentity::new(
            "arn:aws:iam::123456789012:role/WebIdentityRole".to_string(),
            "/path/to/token".into(),
            "https://sts.amazonaws.com".parse().unwrap(),
            "session-name".to_string(),
        );
        assert_eq!(
            provider1, provider2,
            "Providers with same config should be equal"
        );

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            assert_eq!(
                provider1, provider2,
                "Providers should still be equal after async block"
            );
        });
    }
    #[tokio::test]
    pub async fn mock_request() {
        let tmp = std::env::temp_dir();
        let token_file = tmp.join("test_token_file.txt");
        tokio::fs::write(&token_file, "test-web-identity-token")
            .await
            .unwrap();
        let provider = super::AssumeRoleWithWebIdentity::new(
            "arn:aws:iam::123456789012:role/WebIdentityRole".to_string(),
            token_file,
            "https://sts.amazonaws.com".parse().unwrap(),
            "session-name".to_string(),
        );
        let body = Bytes::from(
            quick_xml::se::to_string(&generate_response())
                .unwrap()
                .into_bytes(),
        );
        let response = MockResponse {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: body,
        };
        let mock_client = MockOkClient::new(response);

        let result = provider.provide(mock_client.clone()).await;
        assert!(result.is_ok(), "Mock request should succeed");
        let creds = result.unwrap();
        assert_eq!(creds.access_key, "ASgeIAIOSFODNN7EXAMPLE");
        assert_eq!(
            creds.secret_key,
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYzEXAMPLEKEY"
        );

        // Second call should use cached credentials
        let result = provider.provide(mock_client.clone()).await;
        assert!(result.is_ok(), "Second call should also succeed");
        let creds2 = result.unwrap();
        assert_eq!(creds, creds2, "Credentials should be the same as cached");

        assert_eq!(
            mock_client.request_count(),
            1,
            "Only one request should be made"
        );
    }

    fn generate_response() -> AssumeRoleWithWebIdentityResponse {
        AssumeRoleWithWebIdentityResponse {
            assume_role_with_web_identity_result: AssumeRoleWithWebIdentityResult {
                subject_from_web_identity_token: "amzn1.account.AF6RHO7KZU5XRVQJGXK6HB56KR2A"
                    .to_string(),
                audience: "client.5498841531868486423.1548@apps.example.com".to_string(),
                assumed_role_user: AssumedRoleUser {
                    arn: "arn:aws:sts::123456789012:assumed-role/FederatedWebIdentityRole/app1"
                        .to_string(),
                    assumed_role_id: "AROACLKWSDQRAOEXAMPLE:app1".to_string(),
                },
                credentials: StsResponseCredentials {
                    session_token: "AQoDYXdzEE0a8ANXXXXXXXXNO1ewxE5TijQyp+IEXAMPLE".to_string(),
                    secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYzEXAMPLEKEY".to_string(),
                    expiration: (Local::now() + Duration::hours(1)).fixed_offset(),
                    access_key_id: "ASgeIAIOSFODNN7EXAMPLE".to_string(),
                },
                source_identity: Some("SourceIdentityValue".to_string()),
                provider: ("www.amazon.com".to_string()),
            },
            response_metadata: ResponseMetadata {
                request_id: "ad4156e9-bce1-11e2-82e6-6b6efEXAMPLE".to_string(),
            },
        }
    }
}
