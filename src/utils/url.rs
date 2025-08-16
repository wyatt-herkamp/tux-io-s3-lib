use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use url::Url;

pub const FRAGMENT: &AsciiSet = &CONTROLS
    // URL_RESERVED
    .add(b':')
    .add(b'?')
    .add(b'#')
    .add(b'[')
    .add(b']')
    .add(b'@')
    .add(b'!')
    .add(b'$')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b';')
    .add(b'=')
    // URL_UNSAFE
    .add(b'"')
    .add(b' ')
    .add(b'<')
    .add(b'>')
    .add(b'%')
    .add(b'{')
    .add(b'}')
    .add(b'|')
    .add(b'\\')
    .add(b'^')
    .add(b'`');

pub const FRAGMENT_SLASH: &AsciiSet = &FRAGMENT.add(b'/');
/// Extension for [url::Url] for S3 API
pub trait S3UrlExt {
    /// S3 Canonical Query String
    fn canonical_query_string(&self) -> String;
    /// S3 Canonical URI String
    fn canonical_uri_string(&self) -> String;
}
impl S3UrlExt for Url {
    fn canonical_query_string(&self) -> String {
        let mut keyvalues: Vec<_> = self.query_pairs().collect();
        keyvalues.sort();
        let keyvalues: Vec<String> = keyvalues
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    utf8_percent_encode(k.as_ref(), FRAGMENT_SLASH),
                    utf8_percent_encode(v.as_ref(), FRAGMENT_SLASH)
                )
            })
            .collect();
        keyvalues.join("&")
    }
    fn canonical_uri_string(&self) -> String {
        let decoded = percent_encoding::percent_decode_str(self.path()).decode_utf8_lossy();

        utf8_percent_encode(&decoded, FRAGMENT).to_string()
    }
}
#[cfg(test)]
mod tests {
    use url::Url;

    use crate::utils::url::S3UrlExt;

    #[test]
    fn test_canonical_query_string() {
        let urls = [
            "http://example.com/bucket/list?key=value&other=value2 space",
            "http://example.com/bucket/list?key=value&other=value2+space",
            "http://example.com/bucket/list?key=value&other=value2%20space",
        ];
        for url in urls {
            let url = Url::parse(url).expect(&format!("Failed to parse URL: {url}"));
            assert_eq!(
                url.canonical_query_string(),
                "key=value&other=value2%20space"
            );
        }
    }
}
