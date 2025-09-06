mod tagging;
pub use tagging::*;

use crate::{command::{BucketCommandType, CommandType}, utils::url::S3UrlExt};
#[derive(Debug, Clone)]
pub struct DeleteObject<'request> {
    pub key: &'request str,
    pub version_id: Option<&'request str>,
}
impl CommandType for DeleteObject<'_> {
    fn http_method(&self) -> http::Method {
        http::Method::DELETE
    }

    fn update_url(&self, url: &mut url::Url) -> Result<(), crate::S3Error> {
        url.append_path(self.key.as_ref())?;
        if let Some(version_id) = &self.version_id {
            url.query_pairs_mut().append_pair("versionId", version_id);
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "DeleteObject"
    }
}
impl BucketCommandType for DeleteObject<'_> {}
