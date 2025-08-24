use std::borrow::Cow;

use http::Method;
use tux_io_s3_types::list::ListType;
use url::Url;
pub mod buckets;
use crate::{
    S3Error,
    command::{BucketCommandType, CommandType},
};

#[derive(Debug, Clone)]
pub struct ListObjectsV2<'request> {
    pub prefix: Cow<'request, str>,
    pub continuation_token: Option<Cow<'request, str>>,
    /// Having a delimiter in the request changes how the request works.
    /// No Delimiter will list everything
    ///
    /// With a delimiter will have results with Common Prefixes
    pub delimiter: Option<Cow<'request, str>>,
    pub max_keys: Option<usize>,
    pub start_after: Option<usize>,
    pub fetch_owner: Option<bool>,
}
impl Default for ListObjectsV2<'_> {
    fn default() -> Self {
        Self {
            prefix: Cow::Borrowed(""),
            continuation_token: None,
            delimiter: None,
            max_keys: None,
            start_after: None,
            fetch_owner: None,
        }
    }
}
impl<'request> ListObjectsV2<'request> {
    pub fn with_delimiter<D>(mut self, delimiter: D) -> Self
    where
        D: Into<Cow<'request, str>>,
    {
        self.delimiter = Some(delimiter.into());
        self
    }
    pub fn with_prefix<P>(mut self, prefix: P) -> Self
    where
        P: Into<Cow<'request, str>>,
    {
        self.prefix = prefix.into();
        self
    }
    pub fn with_continuation_token<T>(mut self, continuation_token: T) -> Self
    where
        T: Into<Cow<'request, str>>,
    {
        self.continuation_token = Some(continuation_token.into());
        self
    }
}
impl CommandType for ListObjectsV2<'_> {
    fn name(&self) -> &'static str {
        "ListObjectsV2"
    }
    fn http_method(&self) -> Method {
        Method::GET
    }

    fn update_url(&self, url: &mut Url) -> Result<(), S3Error> {
        url.query_pairs_mut()
            .append_pair("list-type", ListType::Version2.as_ref())
            .append_pair("prefix", &self.prefix);
        if let Some(delimiter) = &self.delimiter {
            url.query_pairs_mut().append_pair("delimiter", delimiter);
        }
        if let Some(continuation_token) = &self.continuation_token {
            url.query_pairs_mut()
                .append_pair("continuation-token", continuation_token);
        }
        if let Some(max_keys) = &self.max_keys {
            url.query_pairs_mut()
                .append_pair("max-keys", &max_keys.to_string());
        }
        if let Some(start_after) = &self.start_after {
            url.query_pairs_mut()
                .append_pair("start-after", &start_after.to_string());
        }
        if let Some(fetch_owner) = &self.fetch_owner {
            url.query_pairs_mut()
                .append_pair("fetch-owner", &fetch_owner.to_string());
        }
        Ok(())
    }
}
impl BucketCommandType for ListObjectsV2<'_> {}

#[cfg(test)]
mod tests {
    use crate::test::init_test_logger;

    use super::*;
    #[test]
    fn url_test() {
        let mut url = url::Url::parse("https://example.com/bucket1/").unwrap();
        let command = ListObjectsV2 {
            prefix: Cow::Borrowed("test/"),
            continuation_token: Some(Cow::Borrowed("token")),
            delimiter: Some(Cow::Borrowed("/")),
            max_keys: Some(100),
            start_after: Some(50),
            fetch_owner: Some(true),
        };
        command.update_url(&mut url).unwrap();
        assert_eq!(
            url.as_str(),
            "https://example.com/bucket1/?list-type=2&prefix=test%2F&delimiter=%2F&continuation-token=token&max-keys=100&start-after=50&fetch-owner=true"
        );
    }
    #[tokio::test]
    async fn request_check() -> anyhow::Result<()> {
        init_test_logger();
        let client = crate::test::create_test_bucket_client();
        let result = client
            .list_objects_v2(ListObjectsV2 {
                prefix: Cow::Borrowed("rfl/"),
                ..Default::default()
            })
            .await?;
        println!("{:#?}", result);

        let result = client
            .list_objects_v2(ListObjectsV2 {
                delimiter: Some(Cow::Borrowed("/")),
                ..Default::default()
            })
            .await?;
        println!("{:#?}", result);
        Ok(())
    }
}
