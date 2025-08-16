use http::header::CONTENT_TYPE;
use http::{HeaderName, HeaderValue};
pub const TEXT_PLAIN_MIME_TYPE: HeaderValue = HeaderValue::from_static("text/plain");
pub const OCTET_STREAM_MIME_TYPE: HeaderValue =
    HeaderValue::from_static("application/octet-stream");
use http::{HeaderMap, header::ToStrError};
pub mod s3_headers;
pub trait HeaderMapS3Ext {
    fn signed_header_string(&self) -> String;

    fn canonical_header_string(&self) -> Result<String, ToStrError>;

    fn signed_and_canonical_header_string(&self) -> Result<(String, String), ToStrError>;
    fn headers_names(&self) -> Vec<&str>;
    fn content_type(&mut self, content_type: HeaderValue);
}
impl HeaderMapS3Ext for HeaderMap {
    fn signed_header_string(&self) -> String {
        let headers = self.headers_names();
        headers.join(";")
    }
    fn canonical_header_string(&self) -> Result<String, ToStrError> {
        let mut keyvalues = vec![];
        for (key, value) in self.iter() {
            keyvalues.push(format!("{}:{}", key.as_str(), value.to_str()?.trim()))
        }
        keyvalues.sort();
        Ok(keyvalues.join("\n"))
    }
    fn headers_names(&self) -> Vec<&str> {
        let mut keys: Vec<_> = self.keys().map(HeaderName::as_str).collect();
        keys.sort();
        keys
    }
    fn signed_and_canonical_header_string(&self) -> Result<(String, String), ToStrError> {
        let mut keys = Vec::with_capacity(self.len());
        let mut key_values = Vec::with_capacity(self.len());
        for (key, value) in self.iter() {
            let key_str = key.as_str().to_lowercase();
            keys.push(key_str);
            key_values.push(format!("{}:{}", key, value.to_str()?.trim()));
        }
        keys.sort();
        key_values.sort();
        let signed_headers = keys.join(";");
        let canonical_headers = key_values.join("\n");
        Ok((canonical_headers, signed_headers))
    }

    fn content_type(&mut self, content_type: HeaderValue) {
        self.insert(CONTENT_TYPE, content_type);
    }
}
