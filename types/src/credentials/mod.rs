use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct Credentials {
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub security_token: Option<String>,
    pub session_token: Option<String>,
    //pub expiration: Option<Rfc3339OffsetDateTime>,
}
impl Credentials {
    pub fn load_from_local() -> Option<Self> {
        let access_key = std::env::var("AWS_ACCESS_KEY_ID").ok();
        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").ok();
        let security_token = std::env::var("AWS_SESSION_TOKEN").ok();
        let credentials = Self {
            access_key,
            secret_key,
            security_token,
            session_token: None,
        };
        Some(credentials)
    }
    pub fn access_key_and_secret(&self) -> Option<(&str, &str)> {
        if let (Some(access_key), Some(secret_key)) = (&self.access_key, &self.secret_key) {
            Some((access_key.as_ref(), secret_key.as_ref()))
        } else {
            None
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
}
impl CredentialsVariants {
    pub fn load_from_environment() -> Option<Self> {
        let access_key = std::env::var("AWS_ACCESS_KEY_ID").ok();
        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").ok();
        if let (Some(access_key), Some(secret_key)) = (access_key, secret_key) {
            Some(Self::AccessAndSecret {
                access_key,
                secret_key,
            })
        } else {
            None
        }
    }
}
