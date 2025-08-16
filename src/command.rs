use http::HeaderMap;
use url::Url;
pub mod body;
pub mod delete;
pub mod get;
pub mod head;
pub mod list;
pub mod put;
use crate::S3Error;
pub use body::S3CommandBody;

pub trait CommandType: Sized {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn http_method(&self) -> http::Method;

    fn metadata_is_invalid(&self) -> bool {
        false
    }
    fn update_url(&self, _url: &mut Url) -> Result<(), S3Error> {
        Ok(())
    }
    fn headers(&self, _base: &mut HeaderMap) -> Result<(), S3Error> {
        Ok(())
    }
    fn into_body(self) -> Result<S3CommandBody, S3Error> {
        Ok(S3CommandBody::default())
    }
}
/// A command that operates on a specific bucket
pub trait BucketCommandType: CommandType {}
/// Represents a command that operates without a specified bucket
///
/// TODO: Find a better name
pub trait AccountCommandType: CommandType {}
