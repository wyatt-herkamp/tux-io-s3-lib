use http::{HeaderValue, Method};
use tux_io_s3_types::headers::{X_AMZ_COPY_SOURCE, X_AMZ_RENAME_SOURCE};
use url::Url;

use crate::command::{BucketCommandType, CommandType};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyObject<'request> {
    pub source: &'request str,
    pub destination: &'request str,
}

impl<'request> CopyObject<'request> {
    pub fn new(source: &'request str, destination: &'request str) -> Self {
        Self {
            source,
            destination,
        }
    }
}

impl Default for CopyObject<'_> {
    fn default() -> Self {
        Self {
            source: "",
            destination: "",
        }
    }
}

impl CommandType for CopyObject<'_> {
    fn http_method(&self) -> http::Method {
        Method::PUT
    }
    fn update_url(&self, url: &mut Url) -> Result<(), crate::S3Error> {
        *url = url.join(self.destination.as_ref())?;
        Ok(())
    }
    fn headers(&self, base: &mut http::HeaderMap) -> Result<(), crate::S3Error> {
        base.insert(X_AMZ_COPY_SOURCE, HeaderValue::from_str(self.source)?);
        Ok(())
    }
}
impl BucketCommandType for CopyObject<'_> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameObject<'request> {
    pub source: &'request str,
    pub destination: &'request str,
}

impl<'request> RenameObject<'request> {
    pub fn new(source: &'request str, destination: &'request str) -> Self {
        Self {
            source,
            destination,
        }
    }
}

impl Default for RenameObject<'_> {
    fn default() -> Self {
        Self {
            source: "",
            destination: "",
        }
    }
}

impl CommandType for RenameObject<'_> {
    fn name(&self) -> &'static str {
        "RenameObject"
    }
    fn http_method(&self) -> http::Method {
        Method::PUT
    }
    fn update_url(&self, url: &mut Url) -> Result<(), crate::S3Error> {
        *url = url.join(self.destination.as_ref())?;
        url.query_pairs_mut().append_key_only("rename");
        Ok(())
    }
    fn headers(&self, base: &mut http::HeaderMap) -> Result<(), crate::S3Error> {
        base.insert(X_AMZ_RENAME_SOURCE, HeaderValue::from_str(self.source)?);
        Ok(())
    }
}
impl BucketCommandType for RenameObject<'_> {}
