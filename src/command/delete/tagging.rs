use http::Method;

use crate::{
    S3Error,
    command::{BucketCommandType, CommandType},
    utils::url::S3UrlExt,
};

#[derive(Debug, Clone)]
pub struct DeleteObjectTagging<'request> {
    pub key: &'request str,
    pub version_id: Option<&'request str>,
}
impl<'request> From<&'request str> for DeleteObjectTagging<'request> {
    fn from(key: &'request str) -> Self {
        Self {
            key: key,
            version_id: None,
        }
    }
}
impl Default for DeleteObjectTagging<'static> {
    fn default() -> Self {
        Self {
            key: "",
            version_id: None,
        }
    }
}
impl CommandType for DeleteObjectTagging<'_> {
    fn name(&self) -> &'static str {
        "DeleteObjectTagging"
    }
    fn http_method(&self) -> Method {
        Method::DELETE
    }
    fn update_url(&self, url: &mut url::Url) -> Result<(), S3Error> {
        url.append_path(self.key.as_ref())?;
        url.query_pairs_mut().append_key_only("tagging");
        if let Some(version_id) = &self.version_id {
            url.query_pairs_mut().append_pair("versionId", version_id);
        }
        Ok(())
    }
}
impl BucketCommandType for DeleteObjectTagging<'_> {}
#[cfg(test)]
mod tests {
    #[test]
    fn url_test() {
        use super::*;
        use url::Url;

        let mut url = Url::parse("https://example.com/bucket1/").unwrap();
        let command = DeleteObjectTagging {
            key: "test.txt",
            ..Default::default()
        };
        command.update_url(&mut url).unwrap();
        assert_eq!(url.as_str(), "https://example.com/bucket1/test.txt?tagging");
    }
}
