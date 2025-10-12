use std::fmt::Display;

pub use http;
use http::{HeaderName, HeaderValue, header::InvalidHeaderValue};
use thiserror::Error;
use tux_io_s3_types::S3ContentError;

use crate::{
    client::HttpResponseError,
    credentials::{error::SigningRelatedError, provider::CredentialsProviderError},
};
pub use tux_io_s3_types as types;
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
    ContentError(#[from] S3ContentError),
    #[error(transparent)]
    SigningRelatedError(#[from] SigningRelatedError),
    #[error(transparent)]
    HttpError(Box<HttpResponseError>),
    #[error("Chunk Must be atleast 8KB")]
    ChunkTooSmall(usize),
    #[error(transparent)]
    CredentialsError(#[from] CredentialsProviderError),
    #[error("Error Reading Body From Stream")]
    BodyReadError(Box<dyn std::error::Error + Send + Sync>),
}
impl S3Error {
    /// Returns the HTTP Status Code Related to this error if applicable.
    pub fn status_code(&self) -> Option<http::StatusCode> {
        match self {
            S3Error::HttpError(err) => err.status_code(),
            S3Error::CredentialsError(err) => err.status_code(),
            _ => None,
        }
    }
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
