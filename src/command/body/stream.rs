use bytes::{Bytes, BytesMut};
use chrono::{DateTime, Utc};
use futures::Stream;
use pin_project::pin_project;
use std::{
    error::Error,
    pin::Pin,
    task::{Context, Poll},
};
use tux_io_s3_types::Service;

use crate::{
    EMPTY_HASH, S3Error,
    command::body::create_payload_signature,
    credentials::{error::SigningRelatedError, sha256_from_bytes, sign_content},
    utils::stream::MinimumSizedStream,
};
#[pin_project]
pub struct S3ContentStream<
    E: Into<Box<dyn Error + Send + Sync>>,
    S: Stream<Item = Result<Bytes, E>>,
> {
    #[pin]
    pub stream: MinimumSizedStream<E, S>,
    pub time: DateTime<Utc>,
    pub previous_signature: String,
    pub region: String,
    pub service: Service,
    pub signing_key: Vec<u8>,
    pub sent_final_chunk: bool,
}
impl<E: Into<Box<dyn Error + Send + Sync>>, S: Stream<Item = Result<Bytes, E>>>
    S3ContentStream<E, S>
{
    pub fn new(
        stream: MinimumSizedStream<E, S>,
        time: DateTime<Utc>,
        previous_signature: String,
        region: String,
        service: Service,
        signing_key: Vec<u8>,
    ) -> Self {
        Self {
            stream,
            time,
            previous_signature,
            region,
            service,
            signing_key,
            sent_final_chunk: false,
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum S3ContentStreamError {
    #[error(transparent)]
    InternalError(Box<dyn Error + Send + Sync>),
    #[error(transparent)]
    S3Error(S3Error),
    #[error(transparent)]
    SigningRelatedError(SigningRelatedError),
}

impl<E: Into<Box<dyn Error + Send + Sync>>, S: Stream<Item = Result<Bytes, E>>> Stream
    for S3ContentStream<E, S>
{
    type Item = Result<Bytes, S3ContentStreamError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.stream.poll_next(cx) {
            Poll::Ready(Some(Ok(data))) => {
                let hash = sha256_from_bytes(&data);
                let content = create_payload_signature(
                    *this.time,
                    this.previous_signature,
                    &this.region,
                    &this.service,
                    &hash,
                );
                let signature = match sign_content(&content, this.signing_key) {
                    Ok(ok) => ok,
                    Err(err) => {
                        return Poll::Ready(Some(Err(S3ContentStreamError::SigningRelatedError(
                            err,
                        ))));
                    }
                };
                let mut actual_content = BytesMut::with_capacity(data.len() + 64);
                actual_content
                    .extend_from_slice(format!("{:x};chunk-signature=", data.len()).as_bytes());
                actual_content.extend_from_slice(signature.as_bytes());
                actual_content.extend_from_slice(b"\r\n");
                actual_content.extend_from_slice(data.as_ref());
                actual_content.extend_from_slice(b"\r\n");
                *this.previous_signature = signature;
                Poll::Ready(Some(Ok(actual_content.freeze())))
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(S3ContentStreamError::InternalError(e.into()))))
            }
            Poll::Ready(None) => {
                if !*this.sent_final_chunk {
                    *this.sent_final_chunk = true;
                    let content = create_payload_signature(
                        *this.time,
                        this.previous_signature,
                        &this.region,
                        &this.service,
                        EMPTY_HASH,
                    );
                    let signature = match sign_content(&content, this.signing_key) {
                        Ok(ok) => ok,
                        Err(err) => {
                            return Poll::Ready(Some(Err(
                                S3ContentStreamError::SigningRelatedError(err),
                            )));
                        }
                    };
                    let content = format!("0;chunk-signature={}\r\n\r\n", signature);
                    let final_chunk = Bytes::from(content);
                    Poll::Ready(Some(Ok(final_chunk)))
                } else {
                    Poll::Ready(None)
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, _) = self.stream.size_hint();
        // We have more content with the chunk headers
        (low, None)
    }
}
