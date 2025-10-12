use std::pin::Pin;

use bytes::{Bytes, BytesMut};
use chrono::{DateTime, Utc};
use futures::{Stream, TryStream, TryStreamExt};
use serde::Serialize;
use tokio::io::AsyncRead;
use tokio_util::io::ReaderStream;
use tracing::trace;
use tux_io_s3_types::{S3ContentError, Service};
mod stream;
use crate::{
    EMPTY_HASH, S3Error,
    credentials::signing::CHRONO_SHORT_DATE_FORMAT,
    utils::{
        LONG_DATE_FORMAT,
        stream::{DynMinSizedStream, MinimumSizedStream, S3_MINIMUM_SIZE, S3_RECOMMENDED_SIZE},
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
    FixedContent(Bytes),
    /// Streaming Data
    Stream {
        /// Must be based on a
        stream: DynMinSizedStream,
        content_length: usize,
    },
    /// Streams that are less than [S3_MINIMUM_SIZE]
    ///
    /// This is because S3 says the minimum chunk size is 8KB and it is probably better to just read this and send it without chunking.
    SmallStream {
        stream: DynStream,
        content_length: usize,
    },
    /// No Body
    None,
}
impl S3CommandBodyInner {
    /// If the variant is SmallStream it reads it into Bytes
    pub(crate) async fn into_fixed_stream(self) -> Result<FixedStream, S3Error> {
        match self {
            S3CommandBodyInner::FixedContent(bytes) => Ok(FixedStream::FixedContent(bytes)),
            S3CommandBodyInner::Stream {
                stream,
                content_length,
            } => Ok(FixedStream::Stream {
                stream,
                content_length,
            }),
            S3CommandBodyInner::None => Ok(FixedStream::None),

            S3CommandBodyInner::SmallStream {
                mut stream,
                content_length,
            } => {
                trace!(?content_length, "Reading SmallStream into Into A Fixed");
                let mut bytes = BytesMut::with_capacity(content_length);
                while let Some(item) = stream.try_next().await.map_err(S3Error::BodyReadError)? {
                    bytes.extend_from_slice(&item);
                }
                let body = bytes.freeze();
                Ok(FixedStream::FixedContent(body))
            }
        }
    }
}
macro_rules! into_fixed_size {
    (
        $(
            $type:ty
        ),*
    ) => {
        $(
            impl From<$type> for S3CommandBody {
                fn from(value: $type) -> Self {
                    S3CommandBody {
                        inner: S3CommandBodyInner::FixedContent(Bytes::from(value)),
                    }
                }
            }
        )*

    };
}
into_fixed_size! {
    String, &'static str, Bytes, Vec<u8>, &'static [u8]
}
pub(crate) enum FixedStream {
    /// Sized is fixed.
    FixedContent(Bytes),
    /// Streaming Data
    Stream {
        /// Must be based on a
        stream: DynMinSizedStream,
        content_length: usize,
    },
    /// No Body
    None,
}
type DynStream =
    Pin<Box<dyn Stream<Item = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send>>;
impl S3CommandBody {
    /// Serializes Content
    pub fn xml_content<S: Serialize>(content: &S) -> Result<Self, S3Error> {
        let xml = quick_xml::se::to_string(content).map_err(S3ContentError::from)?;
        Ok(S3CommandBody {
            inner: S3CommandBodyInner::FixedContent(Bytes::from(xml)),
        })
    }
    // TODO: Replace wrap_stream_with_chunk_size and wrap_reader_with_chunk_size with default a more direct Reader to Stream Impl that handles the size buffering
    /// Wraps a Stream in a [ReaderStream] then passes it into [Self::wrap_stream_with_chunk_size] with [S3_RECOMMENDED_SIZE]
    pub fn wrap_reader<R: AsyncRead + Send + 'static>(stream: R, content_length: usize) -> Self {
        let reader_stream = ReaderStream::new(stream);
        Self::wrap_stream_with_chunk_size(reader_stream, content_length, S3_RECOMMENDED_SIZE)
            .unwrap()
    }
    pub fn wrap_reader_with_chunk_size<R: AsyncRead + Send + 'static>(
        stream: R,
        content_length: usize,
        chunk_size: usize,
    ) -> Result<Self, S3Error> {
        let reader_stream = ReaderStream::new(stream);
        Self::wrap_stream_with_chunk_size(reader_stream, content_length, chunk_size)
    }
    /// Wraps a Stream in a [MinimumSizedStream] and returns an S3CommandBody::Stream.
    ///
    /// The content will be stream
    ///
    /// # Note
    /// If the content length is less than 8KB, it will be read completely into memory and treated like FixedSize.
    pub fn wrap_stream<S>(stream: S, content_length: usize) -> Self
    where
        S: TryStream + Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        Self::wrap_stream_with_chunk_size(stream, content_length, S3_RECOMMENDED_SIZE).unwrap()
    }
    pub fn wrap_stream_with_chunk_size<S>(
        stream: S,
        content_length: usize,
        chunk_size: usize,
    ) -> Result<Self, S3Error>
    where
        S: TryStream + Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        if chunk_size < 8192 {
            return Err(S3Error::ChunkTooSmall(chunk_size));
        }
        let stream = stream.map_err(|e| e.into()).map_ok(Bytes::from);
        let dyn_stream: DynStream = Box::pin(stream);
        let inner = if content_length < S3_MINIMUM_SIZE {
            S3CommandBodyInner::SmallStream {
                stream: dyn_stream,
                content_length,
            }
        } else {
            S3CommandBodyInner::Stream {
                stream: MinimumSizedStream::new(dyn_stream).with_minimum_size(chunk_size),
                content_length,
            }
        };

        let result = S3CommandBody { inner };
        Ok(result)
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
