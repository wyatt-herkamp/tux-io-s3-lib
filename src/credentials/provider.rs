use std::fmt::Debug;

use tux_io_s3_types::credentials::{Credentials, CredentialsVariants};
mod assume_role;
pub use assume_role::*;

use crate::client::http_client::HttpClient;
#[derive(Debug, thiserror::Error)]
pub enum CredentialsProviderError {
    #[error(transparent)]
    AssumeRoleWithWebIdentity(#[from] AssumeRoleWithWebIdentityError),
}
impl CredentialsProviderError {
    pub fn status_code(&self) -> Option<http::StatusCode> {
        match self {
            CredentialsProviderError::AssumeRoleWithWebIdentity(err) => err.status_code(),
        }
    }
}
pub trait CredentialsProviderType: Send + Sync + Debug {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    /// Asynchronously provides credentials.
    ///
    /// The returned future resolves to a `Result` containing the `Credentials` on success,
    fn provide(
        &self,
        client: impl HttpClient,
    ) -> impl std::future::Future<Output = Result<Credentials, CredentialsProviderError>> + Send;
}
impl CredentialsProviderType for Credentials {
    fn name(&self) -> &'static str {
        "StaticCredentialsProvider"
    }
    fn provide(
        &self,
        _client: impl HttpClient,
    ) -> impl std::future::Future<Output = Result<Credentials, CredentialsProviderError>> + Send
    {
        let creds = self.clone();
        async move { Ok(creds) }
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum CredentialsProvider {
    Static(Credentials),
    AssumeRoleWithWebIdentity(AssumeRoleWithWebIdentity),
}
impl CredentialsProviderType for CredentialsProvider {
    fn name(&self) -> &'static str {
        match self {
            CredentialsProvider::Static(creds) => creds.name(),
            CredentialsProvider::AssumeRoleWithWebIdentity(provider) => provider.name(),
        }
    }
    async fn provide(
        &self,
        client: impl HttpClient,
    ) -> Result<Credentials, CredentialsProviderError> {
        match self {
            CredentialsProvider::Static(creds) => creds.provide(client).await,
            CredentialsProvider::AssumeRoleWithWebIdentity(provider) => {
                provider.provide(client).await
            }
        }
    }
}
impl Default for CredentialsProvider {
    fn default() -> Self {
        CredentialsProvider::Static(Credentials::default())
    }
}
impl From<CredentialsVariants> for CredentialsProvider {
    fn from(value: CredentialsVariants) -> Self {
        match value {
            CredentialsVariants::AccessAndSecret {
                access_key,
                secret_key,
            } => CredentialsProvider::Static(Credentials {
                access_key,
                secret_key,
            }),
            CredentialsVariants::AssumeRoleWithWebIdentity {
                role_arn,
                web_identity_token_file,
                sts_endpoint,
                session_name,
            } => {
                let session_name = session_name.unwrap_or_else(|| "aws-creds".to_string());
                CredentialsProvider::AssumeRoleWithWebIdentity(AssumeRoleWithWebIdentity::new(
                    role_arn,
                    web_identity_token_file,
                    sts_endpoint,
                    session_name,
                ))
            }
        }
    }
}
impl From<Credentials> for CredentialsProvider {
    fn from(value: Credentials) -> Self {
        CredentialsProvider::Static(value)
    }
}
impl From<AssumeRoleWithWebIdentity> for CredentialsProvider {
    fn from(value: AssumeRoleWithWebIdentity) -> Self {
        CredentialsProvider::AssumeRoleWithWebIdentity(value)
    }
}
