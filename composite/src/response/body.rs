use std::{collections::VecDeque, convert::Infallible, io::SeekFrom, num::NonZeroU64, pin::Pin, sync::Arc, task::Poll};

use hyper::body::{Bytes, SizeHint};
use tokio::io::{AsyncRead, AsyncSeek, ReadBuf};

use crate::{error::RangeResponseError, parsers::ranges::RangeRequestPart};

use super::ranges::RangeResponseHeader;

// Can't use stdlib's DEFAULT_BUF_SIZE, 16KiB is a guess tbh
const READ_BUFFER_LENGTH: usize = 16384;

/// An empty response body. Common usecases include 204 or 3xx responses
#[derive(Debug)]
pub struct EmptyBody {}
impl hyper::body::Body for EmptyBody {
	type Data = Bytes;
	type Error = Infallible;
	#[inline]
	fn poll_frame(
		self: std::pin::Pin<&mut Self>,
		_: &mut std::task::Context<'_>,
	) -> Poll<Option<Result<hyper::body::Frame<Bytes>, Self::Error>>> {
		Poll::Ready(None)
	}
	#[inline]
	fn is_end_stream(&self) -> bool {
		true
	}
	#[inline]
	fn size_hint(&self) -> SizeHint {
		SizeHint::with_exact(0)
	}
}
impl EmptyBody {
	pub fn new() -> Self {
		Self {}
	}
}

/// Takes a `tokio::io::AsyncRead` stream and uses that for the response body.
pub struct AsyncStreamedBody<R> {
	total_bytes: u64,
	sent_bytes: u64,
	readable: R,
	tmp_bytes: Vec<u8>
}
impl<R> AsyncStreamedBody<R> where R: AsyncRead + Unpin {
	/// Consumes `readable` to use as the response body data.
	/// If you know the total data length of the stream, then `total_bytes` should be specified.
	/// Additionally, if the data stream ends up being larger than `total_bytes`, the data will be truncated.
	pub fn new(readable: R, total_bytes: Option<u64>) -> Self {
		Self {
			total_bytes: total_bytes.unwrap_or(u64::MAX),
			sent_bytes: 0,
			readable,
			tmp_bytes: vec![0; READ_BUFFER_LENGTH]
		}
	}
}

impl<R> hyper::body::Body for AsyncStreamedBody<R> where R: AsyncRead + Unpin {
    type Data = Bytes;
    type Error = std::io::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Bytes>, std::io::Error>>> {
		if self.sent_bytes == self.total_bytes {
			return Poll::Ready(None);
		}

		let mut tmp_bytes = std::mem::take(&mut self.tmp_bytes);
		let remaining_bytes = self.total_bytes.saturating_sub(self.sent_bytes) as usize;
		if remaining_bytes < tmp_bytes.len() {
			tmp_bytes.truncate(remaining_bytes);
		}
		let mut read_buf = ReadBuf::new(&mut tmp_bytes);
		let result = match AsyncRead::poll_read(Pin::new(&mut self.readable), cx, &mut read_buf) {
			Poll::Ready(Ok(_)) => {
				// From<Box<[u8]>> for Bytes is super fast
				let filled: Box<[u8]> = read_buf.filled().into();
				if filled.len() == 0 {
					self.sent_bytes = self.total_bytes;
					return Poll::Ready(None);
				}
				self.sent_bytes += filled.len() as u64;
				Poll::Ready(Some(Ok(hyper::body::Frame::data(filled.into()))))
			},
			Poll::Ready(Err(err)) => {
				Poll::Ready(Some(Err(err)))
			}
			Poll::Pending => {
				Poll::Pending
			},
		};
		// and now that we're done with tmp_bytes, put it back! Woo! No additional heap allocs!
		let _ = std::mem::replace(&mut self.tmp_bytes, tmp_bytes);
		result
    }
	#[inline]
	fn is_end_stream(&self) -> bool {
		self.sent_bytes >= self.total_bytes
	}
	#[inline]
	fn size_hint(&self) -> SizeHint {
		if self.total_bytes == u64::MAX {
			SizeHint::default()
		}else{
			SizeHint::with_exact(self.total_bytes.saturating_sub(self.sent_bytes))
		}
	}
}

/// A body made up of a series of anything that can be converted into `Bytes`
/// This can be a `String`, `Vec<u8>`, `Box<[u8]>`, lots of stuff.
pub struct BytesBody {
	bytes_queue: VecDeque<Bytes>,
	total_bytes: u64
}
impl BytesBody {
	/// Returns a `BytesBody` with a default capacity of 0
	pub fn new_empty() -> Self {
		BytesBody { bytes_queue: VecDeque::new(), total_bytes: 0 }
	}

	/// Returns a `BytesBody` with the capacity specified.
	/// This does **not** mean the total byte capacity of the body, but basically the number of times you intend to
	/// call the `append` method.
	pub fn with_capacity(capacity: usize) -> Self {
		BytesBody { bytes_queue: VecDeque::with_capacity(capacity), total_bytes: 0 }
	}

	/// Returns a `BytesBody` with the specified body.
	/// Note that if you intend to call `append` later on, it may be more performant to use `new_empty` or
	/// `with_capacity` instead.
	pub fn new<B>(bytes: B) -> Self where B: Into<Bytes> {
		let mut bytes_queue = VecDeque::with_capacity(1);
		let bytes = bytes.into();
		let total_bytes = bytes.len() as u64;
		if total_bytes > 0 {
			bytes_queue.push_back(bytes);
		}
		BytesBody {
			bytes_queue,
			total_bytes
		}
	}

	/// Appends to the body
	pub fn append<B>(&mut self, bytes: B) where B: Into<Bytes> {
		let bytes = bytes.into();
		if bytes.len() > 0 {
			self.total_bytes += bytes.len() as u64;
			self.bytes_queue.push_back(bytes);
		}
	}
}

impl hyper::body::Body for BytesBody {
    type Data = Bytes;
    type Error = Infallible;
	#[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Bytes>, Self::Error>>> {
		match self.bytes_queue.pop_front() {
			Some(bytes) => {
				self.total_bytes -= bytes.len() as u64;
				Poll::Ready(Some(Ok(hyper::body::Frame::data(bytes))))
			},
			None => Poll::Ready(None)
		}
    }
	#[inline]
	fn is_end_stream(&self) -> bool {
		self.bytes_queue.is_empty()
	}
	#[inline]
	fn size_hint(&self) -> SizeHint {
		SizeHint::with_exact(self.total_bytes)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
enum MultiRangeBodyState {
	#[default]
	FirstPartPrefix,
	PartPrefix,
	PartBytes,
	WaitForSeek,
	Body(u64),
	Tail,
	End
}
/// This is used to construct a content-type of `multipart/byteranges`
pub struct AsyncStreamedMultiRangeBody<R> {
	readable: R,
	state: MultiRangeBodyState,
	part_prefix: Bytes,
	body_end: Bytes,
	bytes_remaining: u64,
	range_requests: Arc<[RangeResponseHeader]>,
	range_requests_index: usize,
	tmp_bytes: Vec<u8>
}
impl<R> AsyncStreamedMultiRangeBody<R> where R: AsyncRead + AsyncSeek + Unpin {
	/// Consumes `readable` to use as the response body data.
	/// If you know the total data length of the stream, then `total_bytes` should be specified.
	/// Additionally, if the data stream ends up being larger than `total_bytes`, the data will be truncated.
	pub fn new(
		readable: R,
		// TODO change this to an enum that's like Known(u64), Autodiscover, Unknown (no don't that's the response factory's job)
		total_size: Option<NonZeroU64>,
		range_requests: Arc<[RangeRequestPart]>,
		seperator: &str,
		content_type: &str
	) -> Result<Self, RangeResponseError> {
		let part_prefix = Bytes::from(
			format!("\r\n--{}\r\nContent-Type: {}\r\nContent-Range: bytes ", seperator, content_type)
		);
		let body_end = Bytes::from(format!("\r\n--{}--\r\n", seperator));
		let range_requests = range_requests.iter().map(|range_request| {
			RangeResponseHeader::new(range_request, total_size)
		}).collect::<Result<Arc<[_]>, _>>()?;

		let mut bytes_remaining = 0;
		for range_request in range_requests.iter() {
			bytes_remaining += part_prefix.len() as u64;
			bytes_remaining += range_request.str_len() as u64 + 4; // "\r\n\r\n" after multipart headers
			bytes_remaining += range_request.range_len().get();
		}
		bytes_remaining += body_end.len() as u64;
		bytes_remaining -= 2; // The extra \r\n counted at the beginning
		Ok(
			Self {
				readable,
				state: MultiRangeBodyState::default(),
				body_end,
				part_prefix,
				bytes_remaining,
				range_requests,
				range_requests_index: 0,
				tmp_bytes: vec![0; READ_BUFFER_LENGTH]
			}
		)
	}
	pub fn bytes_remaining(&self) -> u64 {
		self.bytes_remaining
	}
}

impl<R> hyper::body::Body for AsyncStreamedMultiRangeBody<R> where R: AsyncRead + AsyncSeek + Unpin {
    type Data = Bytes;
    type Error = std::io::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Bytes>, std::io::Error>>> {
		match self.state {
			MultiRangeBodyState::FirstPartPrefix => {
				self.bytes_remaining -= self.part_prefix.len() as u64;
				self.bytes_remaining += 2;

				self.state = MultiRangeBodyState::PartBytes;
				Poll::Ready(Some(Ok(hyper::body::Frame::data(self.part_prefix.slice(2..)))))
			},
			MultiRangeBodyState::PartPrefix => {
				self.bytes_remaining -= self.part_prefix.len() as u64;

				self.state = MultiRangeBodyState::PartBytes;
				Poll::Ready(Some(Ok(hyper::body::Frame::data(self.part_prefix.clone()))))
			},
			MultiRangeBodyState::PartBytes => {
				let range = self.range_requests[self.range_requests_index];
				let range_bytes = Bytes::from(
					format!("{}\r\n\r\n", range)
				);
				self.bytes_remaining -= range_bytes.len() as u64;
				let result = match AsyncSeek::start_seek(
					Pin::new(&mut self.readable), SeekFrom::Start(range.start())
				) {
					Ok(_) => {
						Poll::Ready(Some(Ok(hyper::body::Frame::data(range_bytes))))
					},
					Err(err) => {
						Poll::Ready(Some(Err(err)))
					},
				};
				self.state = MultiRangeBodyState::WaitForSeek;
				result
			},
			MultiRangeBodyState::WaitForSeek => {
				match AsyncSeek::poll_complete(Pin::new(&mut self.readable), cx) {
					Poll::Ready(_) => {
						// if we wanted to be paranoid, we could check if the resulting seek matches the start pos
						self.state = MultiRangeBodyState::Body(
							self.range_requests[self.range_requests_index].range_len().get()
						);
						self.tmp_bytes.resize(READ_BUFFER_LENGTH, 0);
						// Immediately start reading if we can
						self.poll_frame(cx)
					},
					Poll::Pending => Poll::Pending
				}
			},
			MultiRangeBodyState::Body(mut chunk_remaining) => {
				let mut tmp_bytes = std::mem::take(&mut self.tmp_bytes);
				if chunk_remaining < tmp_bytes.len() as u64 {
					tmp_bytes.truncate(chunk_remaining as usize);
				}
				let mut read_buf: ReadBuf<'_> = ReadBuf::new(&mut tmp_bytes);
				let result = match AsyncRead::poll_read(Pin::new(&mut self.readable), cx, &mut read_buf) {
					Poll::Ready(Ok(_)) => {
						// From<Box<[u8]>> for Bytes is super fast
						let filled: Box<[u8]> = read_buf.filled().into();
						// tmp_bytes len is truncated, so these operations shouldn't underflow
						chunk_remaining -= filled.len() as u64;
						self.bytes_remaining -= filled.len() as u64;
						Poll::Ready(Some(Ok(hyper::body::Frame::<Bytes>::data(filled.into()))))
					},
					Poll::Ready(Err(err)) => {
						Poll::Ready(Some(Err(err)))
					}
					Poll::Pending => {
						Poll::Pending
					},
				};
				let _ = std::mem::replace(&mut self.tmp_bytes, tmp_bytes);
				if chunk_remaining == 0 {
					self.range_requests_index += 1;
					if self.range_requests_index >= self.range_requests.len() {
						self.state = MultiRangeBodyState::Tail;
					}else{
						self.state = MultiRangeBodyState::PartPrefix;
					}
				}else{
					self.state = MultiRangeBodyState::Body(chunk_remaining);
				}
				result
			},
			MultiRangeBodyState::Tail => {
				self.bytes_remaining -= self.body_end.len() as u64;
				self.state = MultiRangeBodyState::End;
				Poll::Ready(Some(Ok(hyper::body::Frame::data(self.body_end.clone()))))
			},
			MultiRangeBodyState::End => {
				Poll::Ready(None)
			},
		}
    }
	fn is_end_stream(&self) -> bool {
		self.state == MultiRangeBodyState::End
	}
	fn size_hint(&self) -> SizeHint {
		SizeHint::with_exact(self.bytes_remaining)
	}
}
