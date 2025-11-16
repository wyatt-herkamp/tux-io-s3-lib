use http::Method;
use url::Url;

use crate::command::{BucketCommandType, CommandType};

pub struct HeadBucket;

impl CommandType for HeadBucket {
    fn http_method(&self) -> http::Method {
        Method::HEAD
    }
    fn update_url(&self, url: &mut Url) -> Result<(), crate::S3Error> {
        Ok(())
    }
}
impl BucketCommandType for HeadBucket {}

#[cfg(test)]
mod tests {
    #[cfg(feature = "client-testing")]
    mod client_testing {

        use crate::{
            command::head::bucket::HeadBucket,
            test::{create_test_bucket_client, init_test_logger},
        };

        #[tokio::test]
        async fn test_head_object() -> anyhow::Result<()> {
            init_test_logger();
            let client = create_test_bucket_client();

            let client = client.execute_command(HeadBucket).await?;

            for header in client.headers() {
                println!("{}: {:?}", header.0, header.1);
            }
            let body = client.text().await?;
            println!("Body: {}", body);
            Ok(())
        }
    }
}
