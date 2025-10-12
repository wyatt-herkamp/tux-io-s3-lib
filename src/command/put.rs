use std::{borrow::Cow, str::FromStr};

use ahash::AHashMap;
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use tux_io_s3_types::tag::{AnyTaggingSet, TAGGING_HEADER};
mod actions;
use url::Url;
mod tagging;
use crate::{
    S3Error,
    command::{BucketCommandType, CommandType, S3CommandBody},
    utils::{header::HeaderMapS3Ext, url::S3UrlExt},
};
mod multipart;
pub use actions::*;
pub use multipart::*;
pub use tagging::*;
pub static AMZ_METADATA_PREFIX: &str = "x-amz-meta-";
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AmzMetadataName(pub(crate) HeaderName);
impl AsRef<HeaderName> for AmzMetadataName {
    fn as_ref(&self) -> &HeaderName {
        &self.0
    }
}
impl TryFrom<&str> for AmzMetadataName {
    type Error = S3Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let header = if !value.starts_with("x-amz-meta-") {
            HeaderName::from_str(&format!("x-amz-meta-{}", value))?
        } else {
            HeaderName::from_str(value)?
        };
        Ok(Self(header))
    }
}
impl TryFrom<HeaderName> for AmzMetadataName {
    type Error = S3Error;
    fn try_from(value: HeaderName) -> Result<Self, Self::Error> {
        if !value.as_str().starts_with("x-amz-meta-") {
            let header = HeaderName::from_str(&format!("x-amz-meta-{}", value.as_str()))?;
            return Ok(Self(header));
        }
        Ok(Self(value))
    }
}
impl From<AmzMetadataName> for HeaderName {
    fn from(value: AmzMetadataName) -> Self {
        value.0
    }
}
#[derive(Debug, Clone)]
pub struct PutHeaders {
    pub content_type: Cow<'static, str>,
    pub metadata: AHashMap<AmzMetadataName, HeaderValue>,
}
impl Default for PutHeaders {
    fn default() -> Self {
        Self {
            content_type: "application/octet-stream".into(),
            metadata: AHashMap::default(),
        }
    }
}
impl PutHeaders {
    pub fn new(content_type: impl Into<Cow<'static, str>>) -> Self {
        Self {
            content_type: content_type.into(),
            metadata: AHashMap::default(),
        }
    }
    pub fn with_content_type(mut self, content_type: impl Into<Cow<'static, str>>) -> Self {
        self.content_type = content_type.into();
        self
    }
    pub fn add_metadata(&mut self, name: AmzMetadataName, value: HeaderValue) {
        // During Debug Builds Validate S3 Header Names to not match an existing Amazon Header
        #[cfg(debug_assertions)]
        {
            assert!(
                tux_io_s3_types::headers::is_s3_header(&name.0),
                "Invalid S3 header: {}",
                name.0
            )
        }
        self.metadata.insert(name, value);
    }
}
pub struct PutObject<'request> {
    pub key: &'request str,
    pub tags: Option<AnyTaggingSet<'request>>,
    pub content: S3CommandBody,
    pub headers: PutHeaders,
}
impl CommandType for PutObject<'_> {
    fn http_method(&self) -> Method {
        Method::PUT
    }
    fn update_url(&self, url: &mut Url) -> Result<(), S3Error> {
        url.append_path(self.key.as_ref())?;
        Ok(())
    }
    fn headers(&self, base: &mut HeaderMap) -> Result<(), S3Error> {
        base.content_type(self.headers.content_type.parse()?);
        if let Some(tags) = &self.tags {
            base.insert(TAGGING_HEADER, tags.to_header_value()?);
        }
        for (name, value) in &self.headers.metadata {
            base.insert(name.0.clone(), value.clone());
        }
        Ok(())
    }
    fn into_body(self) -> Result<super::S3CommandBody, S3Error> {
        Ok(self.content)
    }
}

impl BucketCommandType for PutObject<'_> {}
#[cfg(test)]
mod test {
    #[cfg(feature = "client-testing")]
    mod client_testing {

        use bytes::Bytes;
        use tux_io_s3_types::tag::{AnyTaggingSet, BorrowedTag, BorrowedTaggingSet};

        use crate::{
            command::{
                S3CommandBody,
                put::{PutHeaders, PutObject},
            },
            test::{create_test_bucket_client, init_test_logger},
        };

        #[tokio::test]
        async fn test_file_upload() -> anyhow::Result<()> {
            init_test_logger();
            let client = create_test_bucket_client();
            let path = "test-file.txt";

            let content = Bytes::from_static(b"This is a test file content.");
            let borrowed_tags = BorrowedTaggingSet::new(vec![
                BorrowedTag::from(("key1", "value1")),
                BorrowedTag::from(("key2", "value2")),
            ]);
            let put_object = PutObject {
                key: path,
                tags: Some(AnyTaggingSet::Borrowed(borrowed_tags)),
                content: S3CommandBody::from(content),
                headers: PutHeaders {
                    content_type: "text/plain".into(),
                    ..Default::default()
                },
            };

            let response = client.execute_command(put_object).await?;

            assert!(
                response.status().is_success(),
                "Failed to upload file: {}",
                response.text().await?
            );

            Ok(())
        }

        #[tokio::test]
        async fn test_raw_file() -> anyhow::Result<()> {
            let file = tokio::fs::File::open(
                "/media/Other/PersonalProjects/RustProjects/rust-tests/test_file.txt",
            )
            .await?;
            let file_size = file.metadata().await?.len();
            let stream = tokio_util::io::ReaderStream::new(file);
            let body = S3CommandBody::wrap_stream(stream, file_size as usize);
            init_test_logger();
            let client = create_test_bucket_client();
            let path = "test-file.txt";

            let borrowed_tags = BorrowedTaggingSet::new(vec![
                BorrowedTag::from(("key1", "value1")),
                BorrowedTag::from(("key2", "value2")),
            ]);
            let put_object = PutObject {
                key: path,
                tags: Some(AnyTaggingSet::Borrowed(borrowed_tags)),
                content: body,
                headers: PutHeaders {
                    content_type: "text/plain".into(),
                    ..Default::default()
                },
            };

            let response = client.execute_command(put_object).await?;

            assert!(
                response.status().is_success(),
                "Failed to upload file: {}",
                response.text().await?
            );

            Ok(())
        }
    }
}
