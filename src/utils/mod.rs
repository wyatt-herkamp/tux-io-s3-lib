use http::HeaderValue;
pub mod header;
pub mod stream;
pub mod url;

pub static LONG_DATE_FORMAT: &str = "%Y%m%dT%H%M%SZ";
pub const XML_HEADER_VALUE_WITH_CHARSET: HeaderValue =
    HeaderValue::from_static("application/xml; charset=utf-8");

pub const XML_HEADER_VALUE: HeaderValue = HeaderValue::from_static("application/xml");
