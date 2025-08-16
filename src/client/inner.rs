use tokio::sync::RwLock;
use tux_io_s3_types::{credentials::Credentials, region::S3Region};

use crate::client::settings::AccessType;
#[derive(Debug)]
pub(crate) struct S3ClientInner {
    pub(crate) http_client: reqwest::Client,
    pub(crate) region: S3Region,
    /// Should always be true for custom s3 clients.
    ///
    pub(crate) access_type: AccessType,
    pub(crate) credentials: RwLock<Credentials>,
}
impl S3ClientInner {
    pub async fn change_credentials(&self, credentials: Credentials) {
        let mut lock = self.credentials.write().await;
        *lock = credentials;
    }
}
