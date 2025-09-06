use url::Url;

use crate::{S3Error, command::list::ListObjectsV2};

pub trait ListExtension: Default {
    fn validate(&self, _list_objects: &ListObjectsV2) -> Result<(), S3Error> {
        Ok(())
    }
    fn update_url(&self, url: &mut Url) -> Result<(), S3Error>;
}
impl ListExtension for () {
    fn update_url(&self, _url: &mut Url) -> Result<(), S3Error> {
        Ok(())
    }
}

/// Ceph Specific List Extension
/// https://docs.ceph.com/en/latest/radosgw/s3/bucketops/#id4
#[derive(Debug, Clone, Default)]
pub struct CephListExtension {
    pub allow_unordered: Option<bool>,
}
impl ListExtension for CephListExtension {
    fn update_url(&self, url: &mut Url) -> Result<(), S3Error> {
        if let Some(allow_unordered) = self.allow_unordered {
            url.query_pairs_mut()
                .append_pair("allow-unordered", allow_unordered.to_string().as_str());
        }
        Ok(())
    }
}
