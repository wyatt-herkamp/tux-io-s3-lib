use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use url::Url;
mod assume_role_with_web_identity;
pub use assume_role_with_web_identity::*;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct Credentials {
    /// AWS Access Key ID
    pub access_key: String,
    /// AWS Secret Access Key
    pub secret_key: String,
}
impl From<&StsResponseCredentials> for Credentials {
    fn from(value: &StsResponseCredentials) -> Self {
        Self {
            access_key: value.access_key_id.clone(),
            secret_key: value.secret_access_key.clone(),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CredentialsVariants {
    AccessAndSecret {
        access_key: String,
        secret_key: String,
    },
    AssumeRoleWithWebIdentity {
        role_arn: String,
        session_name: Option<String>,
        web_identity_token_file: PathBuf,
        sts_endpoint: Url,
    },
}
impl CredentialsVariants {
    pub fn load_from_environment() -> Option<Self> {
        let access_key = std::env::var("AWS_ACCESS_KEY_ID").ok();
        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").ok();
        if let (Some(access_key), Some(secret_key)) = (access_key, secret_key) {
            return Some(Self::AccessAndSecret {
                access_key,
                secret_key,
            });
        }
        let role_arn = std::env::var("AWS_ROLE_ARN").ok()?;
        let web_identity_token_file = std::env::var("AWS_WEB_IDENTITY_TOKEN_FILE").ok()?;
        if let (Some(role_arn), Some(web_identity_token_file)) =
            (Some(role_arn), Some(web_identity_token_file))
        {
            let sts_endpoint = std::env::var("AWS_STS_ENDPOINT")
                .unwrap_or_else(|_| "https://sts.amazonaws.com".to_string());
            let sts_endpoint = Url::parse(&sts_endpoint).ok()?;
            return Some(Self::AssumeRoleWithWebIdentity {
                role_arn,
                web_identity_token_file: PathBuf::from(web_identity_token_file),
                sts_endpoint,
                session_name: std::env::var("AWS_SESSION_NAME").ok(),
            });
        }
        None
    }
}
