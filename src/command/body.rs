use std::pin::Pin;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::{Stream, TryStream, TryStreamExt};
use serde::Serialize;
use tux_io_s3_types::{ContentParseError, Service};
mod stream;
use crate::{
    EMPTY_HASH, S3Error,
    credentials::signing::CHRONO_SHORT_DATE_FORMAT,
    utils::{
        LONG_DATE_FORMAT,
        stream::{DynMinSizedStream, MinimumSizedStream},
    },
};
pub use stream::*;
pub struct S3CommandBody {
    pub(crate) inner: S3CommandBodyInner,
}
impl Default for S3CommandBody {
    fn default() -> Self {
        S3CommandBody {
            inner: S3CommandBodyInner::None,
        }
    }
}
pub(crate) enum S3CommandBodyInner {
    /// Sized is fixed.
    FixedSize(Bytes),
    /// Streaming Data
    Stream {
        /// Must be based on a
        stream: DynMinSizedStream,
        content_length: usize,
    },
    /// No Body
    None,
}
impl From<Bytes> for S3CommandBody {
    fn from(value: Bytes) -> Self {
        S3CommandBody {
            inner: S3CommandBodyInner::FixedSize(value),
        }
    }
}
impl From<Vec<u8>> for S3CommandBody {
    fn from(value: Vec<u8>) -> Self {
        S3CommandBody {
            inner: S3CommandBodyInner::FixedSize(Bytes::from(value)),
        }
    }
}
type DynStream =
    Pin<Box<dyn Stream<Item = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send>>;
impl S3CommandBody {
    pub fn xml_content<S: Serialize>(content: &S) -> Result<Self, S3Error> {
        let xml = quick_xml::se::to_string(content).map_err(ContentParseError::from)?;
        Ok(S3CommandBody {
            inner: S3CommandBodyInner::FixedSize(Bytes::from(xml)),
        })
    }
    /// Wraps a Stream in a [MinimumSizedStream] and returns an S3CommandBody::Stream.
    ///
    /// The content will be stream
    pub fn wrap_stream<S>(stream: S, content_length: usize) -> Self
    where
        S: TryStream + Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        let stream = stream.map_err(|e| e.into()).map_ok(Bytes::from);
        let dyn_stream: DynStream = Box::pin(stream);
        let min_stream = MinimumSizedStream::new(dyn_stream);
        S3CommandBody {
            inner: S3CommandBodyInner::Stream {
                stream: min_stream,
                content_length,
            },
        }
    }
}
fn create_payload_signature(
    date_time: DateTime<Utc>,
    previous_signature: &str,
    region: &str,
    service: &Service,
    content_hash: &str,
) -> String {
    let scope = format!(
        "{date}/{region}/{service}/aws4_request",
        date = date_time.format(CHRONO_SHORT_DATE_FORMAT),
        region = region,
        service = service
    );
    let content = format!(
        "AWS4-HMAC-SHA256-PAYLOAD\n{timestamp}\n{scope}\n{previous_signature}\n{empty_payload}\n{content}",
        timestamp = date_time.format(LONG_DATE_FORMAT),
        scope = scope,
        previous_signature = previous_signature,
        empty_payload = EMPTY_HASH,
        content = content_hash
    );
    content
}
