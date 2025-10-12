use http::{
    HeaderName, HeaderValue, Method,
    header::{ACCEPT, CONTENT_LENGTH, RANGE},
};
mod tagging;
use crate::{
    InvalidResponseHeader,
    command::{BucketCommandType, CommandType},
    utils::url::S3UrlExt,
};
pub use tagging::*;
#[derive(Debug, Clone, Copy)]
pub struct Ranged {
    pub start: usize,
    pub end: Option<usize>,
}
impl From<Ranged> for HeaderValue {
    fn from(value: Ranged) -> Self {
        let range = if let Some(end) = value.end {
            format!("bytes={start}-{end}", start = value.start)
        } else {
            format!("bytes={start}", start = value.start)
        };
        HeaderValue::from_str(&range).unwrap()
    }
}
pub struct GetObjectResponse(pub reqwest::Response);
impl From<reqwest::Response> for GetObjectResponse {
    fn from(response: reqwest::Response) -> Self {
        GetObjectResponse(response)
    }
}
impl GetObjectResponse {
    pub fn status(&self) -> http::StatusCode {
        self.0.status()
    }
    pub fn headers(&self) -> &http::HeaderMap {
        self.0.headers()
    }
    fn parse_header<F, T>(
        &self,
        header_name: HeaderName,
        parse_fn: F,
    ) -> Result<Option<T>, InvalidResponseHeader>
    where
        F: Fn(&HeaderValue) -> Result<T, Box<dyn std::error::Error + Send + Sync>>,
    {
        let Some(value) = self.headers().get(&header_name) else {
            return Ok(None);
        };
        parse_fn(value)
            .map(Some)
            .map_err(|source| InvalidResponseHeader {
                name: header_name,
                value: value.clone(),
                source,
            })
    }
    pub fn content_length(&self) -> Result<Option<u64>, InvalidResponseHeader> {
        self.parse_header(CONTENT_LENGTH, |header| {
            let length_str = header.to_str().map_err(Box::new)?;
            Ok(length_str.parse::<u64>().map_err(Box::new)?)
        })
    }
    pub fn content_type(&self) -> Result<Option<String>, InvalidResponseHeader> {
        self.parse_header(http::header::CONTENT_TYPE, |header| {
            Ok(header.to_str().map_err(Box::new)?.into())
        })
    }
}
#[derive(Debug, Clone, Default)]
pub struct GetObject<'request> {
    pub key: &'request str,
    pub accept: Option<HeaderValue>,
    pub ranged: Option<Ranged>,
}
impl CommandType for GetObject<'_> {
    fn http_method(&self) -> http::Method {
        Method::GET
    }
    fn update_url(&self, url: &mut url::Url) -> Result<(), crate::S3Error> {
        url.append_path(self.key.as_ref())?;
        Ok(())
    }
    fn headers(&self, base: &mut http::HeaderMap) -> Result<(), crate::S3Error> {
        if let Some(range) = self.ranged {
            let header: HeaderValue = range.into();
            base.insert(RANGE, header);
        }
        if let Some(accept) = &self.accept {
            base.insert(ACCEPT, accept.clone());
        }
        Ok(())
    }
}
impl BucketCommandType for GetObject<'_> {}
#[cfg(test)]
mod tests {

    #[test]
    fn url_test() {
        use super::*;
        use url::Url;

        let mut url = Url::parse("https://example.com/bucket1/").unwrap();
        let command = GetObject {
            key: "test.txt",
            ..Default::default()
        };
        command.update_url(&mut url).unwrap();
        assert_eq!(url.as_str(), "https://example.com/bucket1/test.txt");
    }
    #[cfg(feature = "client-testing")]
    mod client_tests {
        use crate::{
            command::get::GetObject,
            test::{create_test_bucket_client, init_test_logger},
        };
        #[tokio::test]
        async fn get_object() -> anyhow::Result<()> {
            init_test_logger();
            let key = "test-file.txt";
            let command = GetObject {
                key,
                ..Default::default()
            };
            let bucket_client = create_test_bucket_client();

            let response = bucket_client.execute_command(command).await?;
            assert!(response.status().is_success());
            println!("Response: {:?}", response);
            Ok(())
        }
    }
}
