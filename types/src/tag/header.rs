use headers::Header;
use http::HeaderName;

use crate::tag::{OwnedTag, OwnedTaggingSet};
static TAGGING_HEADER: HeaderName = HeaderName::from_static("x-amz-tagging");
impl Header for OwnedTaggingSet {
    fn name() -> &'static http::HeaderName {
        &TAGGING_HEADER
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        let mut tags = Vec::new();

        for value in values {
            let value_parser = form_urlencoded::parse(value.as_bytes());
            tags.reserve(value_parser.size_hint().0);
            for (key, value) in value_parser {
                tags.push(OwnedTag {
                    key: key.into_owned(),
                    value: value.into_owned(),
                });
            }
        }

        Ok(OwnedTaggingSet::from(tags))
    }

    fn encode<E: Extend<http::HeaderValue>>(&self, values: &mut E) {
        let header_value = self
            .to_header_value()
            .expect("Failed to convert to header value");
        values.extend(std::iter::once(header_value));
    }
}
