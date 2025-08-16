use http::{HeaderMap, Method};
use url::Url;

use crate::{
    command::{BucketCommandType, CommandType},
    utils::header::s3_headers::S3HeadersExt,
};

#[derive(Debug, Clone)]
pub struct HeadObject<'request> {
    pub key: &'request str,
}
impl Default for HeadObject<'_> {
    fn default() -> Self {
        Self { key: "" }
    }
}
impl BucketCommandType for HeadObject<'_> {}
impl CommandType for HeadObject<'_> {
    fn http_method(&self) -> http::Method {
        Method::HEAD
    }
    fn update_url(&self, url: &mut Url) -> Result<(), crate::S3Error> {
        *url = url.join(self.key.as_ref())?;
        Ok(())
    }
}
pub struct HeadObjectResponse(pub reqwest::Response);
impl S3HeadersExt for HeadObjectResponse {
    fn headers(&self) -> &HeaderMap {
        &self.0.headers()
    }
}
#[cfg(test)]
mod tests {
    #[cfg(feature = "client-testing")]
    mod client_testing {

        use crate::{
            test::{create_test_bucket_client, init_test_logger},
            utils::header::s3_headers::S3HeadersExt,
        };

        #[tokio::test]
        async fn test_head_object() -> anyhow::Result<()> {
            init_test_logger();
            let path = "test_file.txt";
            let client = create_test_bucket_client();
            let Some(response) = client.head_object(path).await? else {
                panic!("Expected a response, got None");
            };
            for header in response.headers() {
                println!("{}: {:?}", header.0, header.1);
            }
            Ok(())
        }
    }
}
