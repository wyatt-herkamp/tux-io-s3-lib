use bytes::{Bytes, BytesMut};
use futures::Stream;
use pin_project::pin_project;
use std::{
    error::Error,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::AsyncRead;

pub const S3_MINIMUM_SIZE: usize = 8 * 1000; // 8 KB
pub const S3_RECOMMENDED_SIZE: usize = 64 * 1000; // 64 KB
pub const MAX_CAPACITY: usize = 1024 * 1024; // 1 MB
pub type DynMinSizedStream = MinimumSizedStream<
    Box<dyn Error + Send + Sync>,
    Pin<Box<dyn Stream<Item = Result<Bytes, Box<dyn Error + Send + Sync>>> + Send>>,
>;
/// S3 requires chunks and multipart parts to be a minimum size.
///
/// This is a wrapper around a stream that ensures a minimum size for each chunk or part.
#[pin_project]
pub struct MinimumSizedStream<E, S: Stream<Item = Result<Bytes, E>>> {
    #[pin]
    stream: S,
    /// Overrides the known size of the stream
    ///
    /// This is useful because in some cases we know the size but the underlying stream does not
    known_size: Option<usize>,
    /// Total number of bytes read from the stream
    current_read_bytes: usize,
    /// Minimum size that a poll can return
    minimum_size: usize,
    buffer: BytesMut,
}
impl<E, S: Stream<Item = Result<Bytes, E>>> MinimumSizedStream<E, S> {
    pub fn new(stream: S) -> Self {
        let (low, high) = stream.size_hint();

        let capacity = high
            .unwrap_or_else(|| low.max(S3_MINIMUM_SIZE))
            .min(MAX_CAPACITY);
        let mut result = Self::with_capacity(stream, capacity);
        if Some(low) == high {
            result.set_known_size(low);
        }
        result
    }
    pub fn with_capacity(stream: S, capacity: usize) -> Self {
        Self {
            stream,
            known_size: None,
            current_read_bytes: 0,
            minimum_size: S3_RECOMMENDED_SIZE,
            buffer: BytesMut::with_capacity(capacity),
        }
    }
    pub fn set_minimum_size(&mut self, size: usize) {
        self.minimum_size = size;
    }
    pub fn with_minimum_size(mut self, size: usize) -> Self {
        self.set_minimum_size(size);
        self
    }
    pub fn set_known_size(&mut self, size: usize) {
        self.known_size = Some(size);
    }
    pub fn with_known_size(mut self, size: usize) -> Self {
        self.set_known_size(size);
        self
    }

    pub fn bytes_left(&self) -> Option<usize> {
        self.known_size
            .map(|known_size| known_size - self.current_read_bytes)
    }
}
impl<E, S: Stream<Item = Result<Bytes, E>>> Stream for MinimumSizedStream<E, S> {
    type Item = Result<Bytes, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let minimum_size = *this.minimum_size;

        while this.buffer.len() < minimum_size {
            match this.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(data))) => {
                    this.buffer.extend_from_slice(&data);
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Ready(None) => {
                    if !this.buffer.is_empty() {
                        return Poll::Ready(Some(Ok(this.buffer.split().freeze())));
                    } else {
                        return Poll::Ready(None);
                    }
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
        *this.current_read_bytes += this.buffer.len();
        let result = this.buffer.split().freeze();
        Poll::Ready(Some(Ok(result)))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(size) = self.known_size {
            let size = size - self.current_read_bytes;
            (size, Some(size))
        } else {
            self.stream.size_hint()
        }
    }
}

/// A stream that ensures a minimum size for each chunk or part.
///
/// But sourced from [AsyncRead]
#[pin_project]
pub struct MinimumSizedReaderStream<R: AsyncRead> {
    #[pin]
    reader: R,
    minimum_size: usize,
    current_read_bytes: usize,
    buffer: BytesMut,
    size_hint: (usize, Option<usize>),
}

impl<R: AsyncRead> MinimumSizedReaderStream<R> {
    pub fn new(reader: R) -> Self {
        let capacity = S3_MINIMUM_SIZE;
        Self::with_capacity(reader, capacity)
    }
    pub fn with_capacity(reader: R, capacity: usize) -> MinimumSizedReaderStream<R> {
        Self {
            reader,
            minimum_size: S3_MINIMUM_SIZE,
            buffer: BytesMut::with_capacity(capacity),
            current_read_bytes: 0,
            size_hint: (0, None),
        }
    }
    pub fn with_minimum_size(mut self, minimum_size: usize) -> Self {
        self.minimum_size = minimum_size;
        self
    }
    pub fn with_size_hint(mut self, size_hint: (usize, Option<usize>)) -> Self {
        self.size_hint = size_hint;
        self
    }
    pub fn with_known_size(mut self, size: usize) -> Self {
        self.size_hint = (size, Some(size));
        self
    }
}
impl<R: AsyncRead> Stream for MinimumSizedReaderStream<R> {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!("Implement MinimumSizedReaderStream")
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.size_hint {
            (low, Some(high)) if low == high => {
                let known_size = high - self.current_read_bytes;
                (known_size, Some(known_size))
            }
            (low, Some(high)) => {
                let remaining_high = high.saturating_sub(self.current_read_bytes);
                let remaining_low = low.saturating_sub(self.current_read_bytes);
                (remaining_low, Some(remaining_high))
            }
            (low, None) => {
                let remaining = low.saturating_sub(self.current_read_bytes);
                (remaining, Some(remaining))
            }
        }
    }
}
