use std::borrow::Cow;

use chrono::{DateTime, NaiveDate, Utc};
use http::{HeaderMap, HeaderValue};
use tux_io_s3_types::{
    Service,
    region::{RegionType, S3Region},
};

use crate::{
    credentials::{
        error::SigningRelatedError,
        signing::{CHRONO_SHORT_DATE_FORMAT, CanonicalRequest, SigningKey},
    },
    utils::header::HeaderMapS3Ext,
};
pub struct AWS4HMACSHA256Header<'data> {
    pub access_key: Cow<'data, str>,
    pub date: NaiveDate,
    pub aws_region: Cow<'data, str>,
    /// All the header names in lowercase. Will be concatenated with semicolons.
    pub signed_headers: Vec<Cow<'data, str>>,
    pub signature: SigningKey<'data>,
    pub canonical_request: CanonicalRequest<'data>,
    pub service: Service,
}
impl AWS4HMACSHA256Header<'_> {
    pub fn header_value_content(&self) -> Result<String, SigningRelatedError> {
        let AWS4HMACSHA256Header {
            access_key,
            date,
            aws_region,
            signed_headers,
            signature,
            canonical_request,
            service,
        } = self;
        let signed_headers_str = signed_headers.join(";");
        let date_str = date.format(CHRONO_SHORT_DATE_FORMAT).to_string();
        let credential = format!("{access_key}/{date_str}/{aws_region}/{service}/aws4_request");
        let signature_str = canonical_request.encode(&signature.key()?)?;
        Ok(format!(
            "AWS4-HMAC-SHA256 Credential={credential},SignedHeaders={signed_headers_str},Signature={signature_str}",
        ))
    }
    pub fn header_value(&self) -> Result<HeaderValue, SigningRelatedError> {
        let content = self.header_value_content()?;
        Ok(HeaderValue::from_str(&content)?)
    }
}

pub struct AWS4HMACSHA256HeaderBuilder<'data> {
    pub access_key: Option<Cow<'data, str>>,
    pub date: NaiveDate,
    pub aws_region: Option<Cow<'data, str>>,
    pub signed_headers: Option<Vec<Cow<'data, str>>>,
    pub signature: SigningKey<'data>,
    pub canonical_request: CanonicalRequest<'data>,
    pub service: Service,
}
impl Default for AWS4HMACSHA256HeaderBuilder<'_> {
    fn default() -> Self {
        let now: DateTime<Utc> = Utc::now();
        AWS4HMACSHA256HeaderBuilder {
            access_key: None,
            date: now.date_naive(),
            aws_region: None,
            signed_headers: None,
            signature: SigningKey::default().with_date_time(now),
            canonical_request: CanonicalRequest::default(),
            service: Service::default(),
        }
    }
}
impl<'data> AWS4HMACSHA256HeaderBuilder<'data> {
    pub fn headers(mut self, headers: &'data HeaderMap) -> Self {
        let signed_headers = headers.headers_names();
        self.signed_headers = Some(signed_headers.iter().map(|s| Cow::Borrowed(*s)).collect());
        self.canonical_request.headers = Cow::Borrowed(headers);
        self
    }

    pub fn url(mut self, url: &'data url::Url) -> Self {
        self.canonical_request.url = Cow::Borrowed(url);
        self
    }

    pub fn request_info(mut self, method: http::Method, hash: Cow<'data, str>) -> Self {
        self.canonical_request.method = method;
        self.canonical_request.sha256 = hash;
        self
    }
    pub fn authentication<Key, Secret>(mut self, access_key: Key, secret: Secret) -> Self
    where
        Key: Into<Cow<'data, str>>,
        Secret: Into<Cow<'data, str>>,
    {
        self.access_key = Some(access_key.into());
        self.signature.secret_key = secret.into();
        self
    }
    pub fn region(mut self, region: &'data S3Region) -> Self {
        self.aws_region = Some(Cow::Borrowed(region.name()));
        self.signature.region = Cow::Borrowed(region);
        self.canonical_request.region = Cow::Borrowed(region);
        self
    }

    pub fn date_time(mut self, date_time: DateTime<Utc>) -> Self {
        self.signature.date_time = date_time;
        self.date = date_time.date_naive();
        self.canonical_request.timestamp = date_time;

        self
    }

    pub fn build(self) -> Result<AWS4HMACSHA256Header<'data>, SigningRelatedError> {
        let access_key = self
            .access_key
            .ok_or(SigningRelatedError::MissingBuilderParameter("access_key"))?;
        let aws_region = self
            .aws_region
            .ok_or(SigningRelatedError::MissingBuilderParameter("aws_region"))?;

        let signed_headers =
            self.signed_headers
                .ok_or(SigningRelatedError::MissingBuilderParameter(
                    "signed_headers",
                ))?;

        Ok(AWS4HMACSHA256Header {
            access_key,
            date: self.date,
            aws_region,
            signed_headers,
            signature: self.signature,
            canonical_request: self.canonical_request,
            service: self.service,
        })
    }
}
