use std::borrow::Cow;

use http::{HeaderMap, Method};
use tux_io_s3_types::tag::{AnyTaggingSet, TAGGING_HEADER};
use url::Url;

use crate::{
    S3Error,
    command::{BucketCommandType, CommandType, S3CommandBody, put::PutHeaders},
    utils::{XML_HEADER_VALUE, header::HeaderMapS3Ext, url::S3UrlExt},
};

pub struct CreateMultipartUpload<'request> {
    pub key: &'request str,
    pub tags: Option<AnyTaggingSet<'request>>,
    pub headers: PutHeaders,
}
impl CommandType for CreateMultipartUpload<'_> {
    fn http_method(&self) -> Method {
        Method::POST
    }
    fn update_url(&self, url: &mut Url) -> Result<(), S3Error> {
        *url = url.join(&self.key.as_ref())?;
        url.query_pairs_mut().append_key_only("uploads");
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
}

impl BucketCommandType for CreateMultipartUpload<'_> {}

pub struct PutPart<'request> {
    pub key: &'request str,
    pub part_number: u32,
    pub upload_id: Cow<'request, str>,
    pub content: S3CommandBody,
}
impl CommandType for PutPart<'_> {
    fn http_method(&self) -> Method {
        Method::PUT
    }
    fn update_url(&self, url: &mut Url) -> Result<(), S3Error> {
        url.append_path(&self.key.as_ref())?;

        url.query_pairs_mut()
            .append_pair("partNumber", &self.part_number.to_string())
            .append_pair("uploadId", &self.upload_id);
        Ok(())
    }
    fn headers(&self, _: &mut HeaderMap) -> Result<(), S3Error> {
        Ok(())
    }
    fn into_body(self) -> Result<S3CommandBody, S3Error> {
        Ok(self.content)
    }
}

impl BucketCommandType for PutPart<'_> {}

pub struct CompleteMultipartUpload<'request> {
    pub key: &'request str,
    pub upload_id: Cow<'request, str>,
    pub content: tux_io_s3_types::multi_part::CompleteMultipartUpload,
}
impl CommandType for CompleteMultipartUpload<'_> {
    fn http_method(&self) -> Method {
        Method::POST
    }
    fn update_url(&self, url: &mut Url) -> Result<(), S3Error> {
        url.append_path(&self.key.as_ref())?;
        url.query_pairs_mut()
            .append_pair("uploadId", &self.upload_id);
        Ok(())
    }
    fn headers(&self, header: &mut HeaderMap) -> Result<(), S3Error> {
        header.content_type(XML_HEADER_VALUE);
        Ok(())
    }
    fn into_body(self) -> Result<S3CommandBody, S3Error> {
        S3CommandBody::xml_content(&self.content)
    }
}

impl BucketCommandType for CompleteMultipartUpload<'_> {}

pub struct AbortMultipartUpload<'request> {
    pub key: &'request str,
    pub upload_id: Cow<'request, str>,
}
impl CommandType for AbortMultipartUpload<'_> {
    fn http_method(&self) -> Method {
        Method::DELETE
    }
    fn update_url(&self, url: &mut Url) -> Result<(), S3Error> {
        url.append_path(&self.key.as_ref())?;
        url.query_pairs_mut()
            .append_pair("uploadId", &self.upload_id);
        Ok(())
    }
    fn headers(&self, _base: &mut HeaderMap) -> Result<(), S3Error> {
        Ok(())
    }
}
impl BucketCommandType for AbortMultipartUpload<'_> {}
#[cfg(test)]
mod tests {

    #[cfg(feature = "client-testing")]
    mod client_tests {
        use std::borrow::Cow;

        use futures::TryStreamExt;
        use http::header::ETAG;
        use tux_io_s3_types::{
            multi_part::{InitiateMultipartUploadResult, Part},
            tag::{AnyTaggingSet, BorrowedTag, BorrowedTaggingSet},
        };

        use crate::{
            command::{
                S3CommandBody,
                put::{CreateMultipartUpload, PutHeaders, PutPart},
            },
            test::{create_test_bucket_client, init_test_logger},
            utils::stream::MinimumSizedStream,
        };

        #[tokio::test]
        async fn test_multipart_upload() -> anyhow::Result<()> {
            init_test_logger();
            let file = tokio::fs::File::open(
                "/media/Other/PersonalProjects/RustProjects/rust-tests/test_file.txt",
            )
            .await?;
            let stream = tokio_util::io::ReaderStream::new(file);
            let mut min = MinimumSizedStream::with_capacity(stream, 1024 * 1024)
                .with_known_size(5 * 1024 * 1024);
            let client = create_test_bucket_client();
            let key = "test-file-multipart.txt";
            let borrowed_tags = BorrowedTaggingSet::new(vec![
                BorrowedTag::from(("key1", "value1")),
                BorrowedTag::from(("key2", "value2")),
            ]);
            let create = CreateMultipartUpload {
                key,
                tags: Some(AnyTaggingSet::Borrowed(borrowed_tags)),
                headers: PutHeaders {
                    content_type: "text/plain".into(),
                    ..Default::default()
                },
            };

            let response = client.execute_command(create).await?;
            assert!(
                response.status().is_success(),
                "Failed to create multipart upload: {:?}",
                response
            );

            let initate: InitiateMultipartUploadResult =
                quick_xml::de::from_str(&response.text().await?)?;
            let mut parts = Vec::new();
            while let Some(bytes) = min.try_next().await? {
                let part = PutPart {
                    key,
                    part_number: parts.len() as u32 + 1,
                    upload_id: Cow::Borrowed(&initate.upload_id),
                    content: S3CommandBody::from(bytes),
                };
                let response = client.execute_command(part).await?;
                assert!(
                    response.status().is_success(),
                    "Failed to upload part: {:?}",
                    response
                );
                let etag = response
                    .headers()
                    .get(ETAG)
                    .expect("Missing ETag header")
                    .to_str()?
                    .to_owned();
                parts.push(Part {
                    number: parts.len() as u32 + 1,
                    etag,
                });
            }

            let complete = crate::command::put::CompleteMultipartUpload {
                key,
                upload_id: Cow::Borrowed(&initate.upload_id),
                content: tux_io_s3_types::multi_part::CompleteMultipartUpload { parts },
            };
            let response = client.execute_command(complete).await?;
            assert!(
                response.status().is_success(),
                "Failed to complete multipart upload: {:?}",
                response
            );
            Ok(())
        }
    }
}
