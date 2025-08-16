use std::borrow::Cow;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::credentials::error::SigningRelatedError;
use crate::utils::LONG_DATE_FORMAT;
use crate::utils::header::HeaderMapS3Ext;
use crate::utils::url::S3UrlExt;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::Utc;
use hmac::{Hmac, Mac};
use http::HeaderMap;
use http::Method;
use sha2::Digest;
use sha2::Sha256;
use tux_io_s3_types::Service;
use tux_io_s3_types::region::RegionType;
use tux_io_s3_types::region::S3Region;
use url::Url;

pub type HmacSha256 = Hmac<Sha256>;
pub static CHRONO_SHORT_DATE_FORMAT: &str = "%Y%m%d";
pub struct ScopeString<'request> {
    pub date: NaiveDate,
    pub region: &'request S3Region,
    pub service: Service,
}
impl Display for ScopeString<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{date}/{region}/{service}/aws4_request",
            date = self.date.format(CHRONO_SHORT_DATE_FORMAT),
            region = self.region.name(),
            service = self.service
        )
    }
}
#[derive(Debug, Clone, Default)]
pub struct SigningKey<'request> {
    pub secret_key: Cow<'request, str>,
    pub date_time: DateTime<Utc>,
    pub region: Cow<'request, S3Region>,
    pub service: Service,
}
impl SigningKey<'_> {
    pub fn with_date_time(mut self, date_time: DateTime<Utc>) -> Self {
        self.date_time = date_time;
        self
    }
    pub fn key(&self) -> Result<Vec<u8>, SigningRelatedError> {
        let secret = format!("AWS4{}", self.secret_key);
        let mut date_hmac = HmacSha256::new_from_slice(secret.as_bytes())?;
        date_hmac.update(
            self.date_time
                .format(CHRONO_SHORT_DATE_FORMAT)
                .to_string()
                .as_bytes(),
        );
        let mut region_hmac = HmacSha256::new_from_slice(&date_hmac.finalize().into_bytes())?;
        region_hmac.update(self.region.name().as_bytes());
        let mut service_hmac = HmacSha256::new_from_slice(&region_hmac.finalize().into_bytes())?;
        service_hmac.update(self.service.as_bytes());
        let mut signing_hmac = HmacSha256::new_from_slice(&service_hmac.finalize().into_bytes())?;
        signing_hmac.update(b"aws4_request");
        Ok(signing_hmac.finalize().into_bytes().to_vec())
    }
}
#[derive(Debug, Clone)]
pub struct CanonicalRequest<'request> {
    pub method: Method,
    pub url: Cow<'request, Url>,
    pub sha256: Cow<'request, str>,
    pub headers: Cow<'request, HeaderMap>,
    pub timestamp: DateTime<Utc>,
    pub region: Cow<'request, S3Region>,
    pub service: Service,
}
impl Default for CanonicalRequest<'_> {
    fn default() -> Self {
        CanonicalRequest {
            method: Method::GET,
            url: Cow::Owned(Url::parse("https://example.com").unwrap()),
            sha256: Cow::Borrowed(""),
            headers: Cow::Owned(HeaderMap::new()),
            timestamp: Utc::now(),
            region: Cow::Owned(S3Region::default()),
            service: Service::default(),
        }
    }
}
impl CanonicalRequest<'_> {
    pub fn encode(&self, signing_key: &[u8]) -> Result<String, SigningRelatedError> {
        let canonical_request = self.content_ready_for_signing()?;
        let mut hmac = HmacSha256::new_from_slice(signing_key)?;
        hmac.update(canonical_request.as_bytes());
        let signature = hmac.finalize().into_bytes();
        Ok(hex::encode(signature))
    }
    pub fn content_ready_for_signing(&self) -> Result<String, SigningRelatedError> {
        let scope = format!(
            "{date}/{region}/s3/aws4_request",
            date = self.timestamp.format(CHRONO_SHORT_DATE_FORMAT),
            region = self.region.name()
        );
        let content = self.content()?;
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let canonical_request_content_hash = hex::encode(hasher.finalize());
        let actual_content = format!(
            "AWS4-HMAC-SHA256\n{timestamp}\n{scope}\n{hash}",
            timestamp = self.timestamp.format(LONG_DATE_FORMAT),
            scope = scope,
            hash = canonical_request_content_hash
        );

        Ok(actual_content)
    }
    pub fn content(&self) -> Result<String, SigningRelatedError> {
        let CanonicalRequest {
            method,
            url,
            sha256,
            headers,
            ..
        } = self;
        let (canonical_headers, signed_headers) = headers.signed_and_canonical_header_string()?;
        let canonical_uri = url.as_ref().canonical_uri_string();
        let query_string = url.as_ref().canonical_query_string();
        let result = format!(
            "{method}\n{uri}\n{query_string}\n{headers}\n\n{signed}\n{sha256}",
            method = method,
            uri = canonical_uri,
            query_string = query_string,
            headers = canonical_headers,
            signed = signed_headers,
            sha256 = sha256,
        );
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use tux_io_s3_types::region::S3Region;
    use url::Url;

    #[test]
    pub fn test() -> anyhow::Result<()> {
        let mut headers = http::HeaderMap::new();
        headers.insert("x-amz-content-sha256", "UNSIGNED-PAYLOAD".parse().unwrap());
        headers.insert("x-amz-date", "20231001T000000Z".parse().unwrap());
        headers.insert("host", "example.com".parse().unwrap());
        let url = Url::parse("https://example.com/path/to/resource?query=string")?;
        let canonical_request = super::CanonicalRequest {
            method: http::Method::GET,
            url: Cow::Owned(url),
            sha256: "UNSIGNED-PAYLOAD".into(),
            headers: Cow::Owned(headers),
            timestamp: chrono::Utc::now(),
            region: Cow::Owned(S3Region::default()),
            service: tux_io_s3_types::Service::S3,
        };
        let content = canonical_request.content()?;
        println!("Canonical Request:\n{}", content);
        Ok(())
    }
    #[test]
    pub fn signing_key_test() -> anyhow::Result<()> {
        let signing_key = super::SigningKey {
            secret_key: Cow::Borrowed("my_secret_key"),
            date_time: chrono::Utc::now(),
            region: Cow::Owned(S3Region::default()),
            ..Default::default()
        };
        let key = signing_key.key()?;
        println!("Signing Key: {:?}", key);
        Ok(())
    }
}
