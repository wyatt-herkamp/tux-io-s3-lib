pub mod credentials;
//pub mod path;
pub mod multi_part;
pub mod region;
pub mod signature;
pub mod tag;
use std::io::BufRead;
pub mod utils;
use thiserror::Error;
use tokio::io::AsyncBufRead;
pub mod headers;
pub mod list;
pub mod owner;
#[derive(Debug, Error)]
pub enum S3ContentError {
    #[error(transparent)]
    QuickXML(#[from] quick_xml::Error),
    #[error(transparent)]
    QuickXMLSerializeError(#[from] quick_xml::se::SeError),
    #[error(transparent)]
    QuickXMLDeserializeError(#[from] quick_xml::de::DeError),
}
pub trait DataExtract {
    fn extract_data<R: BufRead>(reader: &mut R) -> Result<Self, S3ContentError>
    where
        Self: Sized;
}

#[allow(async_fn_in_trait)]
pub trait AsyncDataExtract {
    async fn extract_data<R: AsyncBufRead + Unpin>(reader: &mut R) -> Result<Self, S3ContentError>
    where
        Self: Sized;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Service {
    #[default]
    S3,
}
impl Service {
    pub fn as_str(&self) -> &'static str {
        match self {
            Service::S3 => "s3",
        }
    }
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            Service::S3 => b"s3",
        }
    }
}
impl AsRef<[u8]> for Service {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
impl AsRef<str> for Service {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
