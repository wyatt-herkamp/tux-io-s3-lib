use http::{HeaderMap, HeaderName, HeaderValue, header::CONTENT_LENGTH};
use tux_io_s3_types::headers::X_AMZ_TAGGING_COUNT;

use crate::{InvalidResponseHeader, S3Error, command::put::AmzMetadataName};

pub trait S3HeadersExt {
    fn headers(&self) -> &HeaderMap;
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
        parse_fn(&value)
            .map(Some)
            .map_err(|source| InvalidResponseHeader {
                name: header_name,
                value: value.clone(),
                source,
            })
    }
    fn content_length(&self) -> Result<Option<u64>, InvalidResponseHeader> {
        self.parse_header(CONTENT_LENGTH, |header| {
            let length_str = header.to_str().map_err(Box::new)?;
            Ok(length_str.parse::<u64>().map_err(Box::new)?)
        })
    }
    fn tagging_count(&self) -> Result<Option<u32>, InvalidResponseHeader> {
        self.parse_header(X_AMZ_TAGGING_COUNT, |header| {
            let count_str = header.to_str().map_err(Box::new)?;
            Ok(count_str.parse::<u32>().map_err(Box::new)?)
        })
    }

    fn get_meta_header(&self, key: &str) -> Result<Option<&HeaderValue>, S3Error> {
        let header_name = AmzMetadataName::try_from(key)?;
        Ok(self.headers().get(header_name.as_ref()))
    }
}
