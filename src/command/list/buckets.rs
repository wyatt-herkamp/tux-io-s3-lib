use tux_io_s3_types::region::{RegionType, S3Region};
use url::Url;

use crate::{
    S3Error,
    command::{AccountCommandType, CommandType},
};

pub struct ListBuckets<'request> {
    pub region: &'request S3Region,
    pub continuation_token: Option<String>,
    pub max_buckets: Option<usize>,
    pub prefix: Option<String>,
}

impl CommandType for ListBuckets<'_> {
    fn name(&self) -> &'static str {
        "ListBuckets"
    }

    fn http_method(&self) -> http::Method {
        http::Method::GET
    }
    fn update_url(&self, url: &mut Url) -> Result<(), S3Error> {
        url.query_pairs_mut()
            .append_pair("bucket-region", self.region.name());
        if let Some(prefix) = &self.prefix {
            url.query_pairs_mut().append_pair("prefix", prefix);
        }
        if let Some(token) = &self.continuation_token {
            url.query_pairs_mut()
                .append_pair("continuation-token", token);
        }
        if let Some(max) = self.max_buckets {
            url.query_pairs_mut()
                .append_pair("max-buckets", &max.to_string());
        }
        Ok(())
    }
}
impl AccountCommandType for ListBuckets<'_> {}

#[cfg(test)]
mod tests {
    #[cfg(feature = "client-testing")]
    mod client_testing {
        use crate::test::init_test_logger;

        #[tokio::test]
        async fn request_check() -> anyhow::Result<()> {
            init_test_logger();
            let client = crate::test::create_test_client();
            let response = client.list_my_buckets().await?;
            println!("{:#?}", response);
            Ok(())
        }
    }
}
