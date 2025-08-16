use http::{HeaderMap, Method, header::ACCEPT};

use crate::{
    S3Error, S3Result,
    command::{BucketCommandType, CommandType},
    utils::XML_HEADER_VALUE,
};

#[derive(Debug, Clone)]
pub struct GetObjectTagging<'request> {
    pub key: &'request str,
    pub version_id: Option<String>,
}
impl Default for GetObjectTagging<'static> {
    fn default() -> Self {
        Self {
            key: "",
            version_id: None,
        }
    }
}
impl CommandType for GetObjectTagging<'_> {
    fn name(&self) -> &'static str {
        "GetObjectTagging"
    }
    fn http_method(&self) -> Method {
        Method::GET
    }
    fn update_url(&self, url: &mut url::Url) -> Result<(), S3Error> {
        *url = url.join(self.key.as_ref())?;
        url.query_pairs_mut().append_key_only("tagging");
        if let Some(version_id) = &self.version_id {
            url.query_pairs_mut().append_pair("versionId", version_id);
        }
        Ok(())
    }
    fn headers(&self, headers: &mut HeaderMap) -> S3Result<()> {
        headers.insert(ACCEPT, XML_HEADER_VALUE);
        Ok(())
    }
}
impl BucketCommandType for GetObjectTagging<'_> {}
#[cfg(test)]
mod tests {

    #[test]
    fn url_test() {
        use super::*;
        use url::Url;

        let mut url = Url::parse("https://example.com/bucket1/").unwrap();
        let command = GetObjectTagging {
            key: "test.txt",
            ..Default::default()
        };
        command.update_url(&mut url).unwrap();
        assert_eq!(url.as_str(), "https://example.com/bucket1/test.txt?tagging");
    }
}
