use std::fmt::Display;

pub use http;
use http::{HeaderName, HeaderValue, header::InvalidHeaderValue};
use thiserror::Error;
use tux_io_s3_types::ContentParseError;

use crate::{client::HttpResponseError, credentials::error::SigningRelatedError};
pub use tux_io_s3_types  as types;
pub mod client;
pub mod command;
pub mod credentials;
#[cfg(test)]
pub mod test;
pub mod utils;
pub const EMPTY_HASH: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
pub type S3Result<T> = Result<T, S3Error>;
#[derive(Debug, Error)]
pub enum S3Error {
    #[error("Utf8 decoding error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    InvalidHeader(#[from] InvalidHeaderValue),
    #[error("Invalid header name: {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[error(transparent)]
    URLParseError(#[from] url::ParseError),
    #[error(transparent)]
    ContentError(#[from] ContentParseError),
    #[error(transparent)]
    SigningRelatedError(#[from] SigningRelatedError),
    #[error(transparent)]
    HttpError(Box<HttpResponseError>),
}
impl From<HttpResponseError> for S3Error {
    fn from(error: HttpResponseError) -> Self {
        S3Error::HttpError(Box::new(error))
    }
}
impl From<reqwest::Error> for S3Error {
    fn from(error: reqwest::Error) -> Self {
        let response = HttpResponseError::from(error);
        Self::from(response)
    }
}
#[derive(Debug, Error)]
pub struct InvalidResponseHeader {
    pub name: HeaderName,
    pub value: HeaderValue,
    #[source]
    pub source: Box<dyn std::error::Error + Send + Sync + 'static>,
}
impl Display for InvalidResponseHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Header `{}` has an invalid value: {:?} error: {}",
            self.name, self.value, self.source
        )
    }
}
