//! HTTP Headers for S3
use http::HeaderName;

macro_rules! s3_headers {
    (
        $(
            $(#[$docs:meta])*
            $header:ident => $header_value:literal
        ),* $(,)?
    ) => {
        $(
            $(#[$docs])*
            pub const $header: HeaderName = HeaderName::from_static($header_value);
        )*

        pub fn is_s3_header(name: &HeaderName) -> bool {
            let s3_headers = [
                $(
                    $header,
                )*
            ];
            s3_headers.contains(&name)
        }
    }
}

s3_headers! {
    /// The `x-amz-content-sha256` header
    X_AMZ_CONTENT_SHA256 => "x-amz-content-sha256",
    /// The `x-amz-date` header
    X_AMZ_DATE => "x-amz-date",
    /// The `x-amz-security-token` header
    X_AMZ_SECURITY_TOKEN => "x-amz-security-token",
    /// The `x-amz-user-agent` header
    X_AMZ_USER_AGENT => "x-amz-user-agent",
    /// The `x-amz-request-id` header
    X_AMZ_REQUEST_ID => "x-amz-request-id",
    /// The `x-amz-tagging` header
    X_AMZ_TAGGING => "x-amz-tagging",
    /// The `x-amz-tagging-count` header
    /// Number of tags
    X_AMZ_TAGGING_COUNT => "x-amz-tagging-count",
    /// The `x-amz-decoded-content-length` header
    X_AMZ_DECODED_CONTENT_LENGTH => "x-amz-decoded-content-length",
    /// The `x-amz-rename-source` header
    X_AMZ_RENAME_SOURCE => "x-amz-rename-source",
    /// The `x-amz-copy-source` header
    X_AMZ_COPY_SOURCE => "x-amz-copy-source",



}
