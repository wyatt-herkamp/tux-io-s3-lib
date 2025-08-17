use hmac::digest::InvalidLength;
use http::header::{InvalidHeaderValue, ToStrError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SigningRelatedError {
    #[error(transparent)]
    InvalidLength(#[from] InvalidLength),
    #[error(transparent)]
    ToStrError(#[from] ToStrError),
    #[error(transparent)]
    InvalidHeader(#[from] InvalidHeaderValue),
    #[error("Missing builder parameter: {0}")]
    MissingBuilderParameter(&'static str),
}
