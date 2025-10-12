use http::{HeaderMap, Method};
use tux_io_s3_types::tag::AnyTaggingSet;
use url::Url;

use crate::{
    S3Error,
    command::{BucketCommandType, CommandType, S3CommandBody},
    utils::{XML_HEADER_VALUE_WITH_CHARSET, header::HeaderMapS3Ext, url::S3UrlExt},
};

pub struct PutTagging<'request> {
    pub key: &'request str,
    pub tagging: AnyTaggingSet<'request>,
}
impl<'request> PutTagging<'request> {
    pub fn new(key: &'request str, tagging: impl Into<AnyTaggingSet<'request>>) -> Self {
        Self {
            key,
            tagging: tagging.into(),
        }
    }
}
impl<'request> CommandType for PutTagging<'request> {
    fn name(&self) -> &'static str {
        "PutTagging"
    }
    fn http_method(&self) -> Method {
        Method::PUT
    }
    fn update_url(&self, url: &mut Url) -> Result<(), S3Error> {
        url.append_path(self.key.as_ref())?;
        url.query_pairs_mut().append_key_only("tagging");
        Ok(())
    }
    fn headers(&self, base: &mut HeaderMap) -> Result<(), S3Error> {
        base.content_type(XML_HEADER_VALUE_WITH_CHARSET);
        Ok(())
    }
    fn into_body(self) -> Result<S3CommandBody, S3Error> {
        S3CommandBody::xml_content(&self.tagging)
    }
}
impl BucketCommandType for PutTagging<'_> {}
#[cfg(test)]
mod tests {

    #[cfg(feature = "client-testing")]
    mod client_testing {
        use rand::distr::{Alphanumeric, Distribution};
        use tux_io_s3_types::tag::{BorrowedTag, BorrowedTaggingSet, TagType};

        use crate::{
            command::put::PutTagging,
            test::{create_test_bucket_client, init_test_logger},
        };

        #[tokio::test]
        async fn test_put_tagging() -> anyhow::Result<()> {
            init_test_logger();
            let key = "test_file.txt";
            let random_string = Alphanumeric
                .sample_iter(&mut rand::rng())
                .take(120)
                .map(char::from)
                .collect::<String>();
            let tagging = BorrowedTaggingSet::new(vec![
                BorrowedTag::new("put-tagging", "test"),
                BorrowedTag::new("random_value", random_string.as_str()),
            ]);
            let command = PutTagging::new(key, tagging);
            let client = create_test_bucket_client();
            let response = client.execute_command(command).await?;
            println!("Tagging response: {:?}", response);
            let content = response.text().await?;
            println!("Response content: {}", content);

            let Some(tagging) = client.get_object_tagging(key).await? else {
                panic!("Expected a tagging response, got None");
            };

            println!("GetTagging response: {:?}", tagging);
            assert_eq!(
                tagging.get_tag("put-tagging").map(|tag| tag.value()),
                Some("test"),
                "Expected tag 'put-tagging' to have value 'test'",
            );

            assert_eq!(
                tagging.get_tag("random_value").map(|tag| tag.value()),
                Some(random_string.as_str()),
                "Expected tag 'random_value' to have value '{}'",
                random_string
            );

            Ok(())
        }
    }
}
