// Rust translation of FoundationClasses/TKernel/Standard/Standard_ReadBuffer.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_ReadBuffer`: reads a stream in fixed-size chunks through a 4096-byte cache.
use super::stream_fill::fill_from;
use super::{Failure, IstreamException, IstreamStream, exceptions};
use bytemuck::AnyBitPattern;
use std::io::Read;
mod file_chunks;
#[cfg(test)]
mod istream_chunks;

/// Size of the native `myBuffer` cache.
const CACHE: usize = 4096;

/// Chunked reader over `data_len` bytes; `ReadBuffer::new(12, 4, false)?.read_data_chunk(&mut s)`.
///
/// The cache is refilled with as many whole chunks as fit (or the remaining data), and each
/// call hands out the next chunk. A chunk may extend past the bytes just read (partial
/// payloads); those bytes keep whatever the cache held before, exactly as natively.
pub struct ReadBuffer {
    cache: Box<[u8; CACHE]>,
    position: Option<usize>,
    end: usize,
    data_len: i64,
    data_read: i64,
    chunk_len: usize,
    buffer_len: usize,
}

impl ReadBuffer {
    /// Constructs and initializes; `ReadBuffer::new(4096 * 3, 12, false)?`.
    pub fn new(data_len: i64, chunk_len: usize, is_partial_payload: bool) -> Result<Self, Failure> {
        let mut buffer = Self {
            cache: Box::new([0; CACHE]),
            position: None,
            end: CACHE,
            data_len: 0,
            data_read: 0,
            chunk_len: 0,
            buffer_len: 0,
        };
        buffer.init(data_len, chunk_len, is_partial_payload)?;
        Ok(buffer)
    }

    /// Restarts reading; without a partial payload the length is truncated to whole chunks
    /// (C++ `%`, so negative lengths round toward zero); `buffer.init(100, 8, true)?`.
    ///
    /// A chunk longer than the cache raises Standard_ProgramError after the fields are set,
    /// as natively; a zero chunk (a native division by zero) is refused up front. Either
    /// way the buffer then hands out no chunk.
    pub fn init(&mut self, data_len: i64, chunk_len: usize, partial: bool) -> Result<(), Failure> {
        if chunk_len == 0 {
            self.position = None;
            return Err(exceptions::program_error(
                "Standard_ReadBuffer::Init: chunk size 0; expected 1..=4096 bytes",
            ));
        }
        self.reset(truncated_length(data_len, chunk_len, partial), chunk_len);
        if chunk_len > CACHE {
            self.position = None;
            return Err(exceptions::program_error(
                "Internal error - chunk size is greater then preallocated buffer",
            ));
        }
        Ok(())
    }

    fn reset(&mut self, data_len: i64, chunk_len: usize) {
        self.data_read = 0;
        self.data_len = data_len;
        self.chunk_len = chunk_len;
        self.buffer_len = chunk_len * (CACHE / chunk_len);
        self.end = CACHE;
        self.position = Some(CACHE);
        self.cache.fill(0);
    }

    /// True once exactly the (truncated) data length has been read; `buffer.is_done()`.
    pub fn is_done(&self) -> bool {
        self.data_read == self.data_len
    }

    /// Returns the next chunk, or `None` at the end of data or on a short read, after
    /// which every call returns `None`; `buffer.read_data_chunk(&mut stream)`.
    pub fn read_data_chunk(&mut self, stream: &mut impl Read) -> Option<&[u8]> {
        let start = self.next_chunk(stream)?;
        Some(&self.cache[start..start + self.chunk_len])
    }

    /// Returns the next chunk read as a `T` (the template `ReadChunk<Chunk_T>`), copied
    /// because the chunk may be unaligned; `buffer.read_chunk::<[f32; 3]>(&mut stream)?`.
    /// A `T` larger than one chunk (native out-of-bounds reads) raises Standard_ProgramError.
    pub fn read_chunk<T: AnyBitPattern>(
        &mut self,
        stream: &mut impl Read,
    ) -> Result<Option<T>, Failure> {
        self.check_stream_chunk_size::<T>()?;
        let size = size_of::<T>();
        let chunk = self.read_data_chunk(stream);
        Ok(chunk.map(|bytes| bytemuck::pod_read_unaligned(&bytes[..size])))
    }

    /// Reads a chunk through the exception-mask-aware istream protocol;
    /// `let chunk = buffer.read_data_chunk_istream(&mut stream)?`.
    pub fn read_data_chunk_istream<R: Read>(
        &mut self,
        stream: &mut IstreamStream<R>,
    ) -> Result<Option<&[u8]>, IstreamException> {
        let start = self.next_chunk_istream(stream)?;
        Ok(start.map(|start| &self.cache[start..start + self.chunk_len]))
    }

    /// Reads a typed chunk while returning masked istream failures;
    /// `let value = buffer.read_chunk_istream::<u32, _>(&mut stream)?`.
    pub fn read_chunk_istream<T: AnyBitPattern, R: Read>(
        &mut self,
        stream: &mut IstreamStream<R>,
    ) -> Result<Option<T>, ReadChunkIstreamError> {
        self.check_stream_chunk_size::<T>()?;
        let chunk = self.read_data_chunk_istream(stream)?;
        Ok(chunk.map(|bytes| bytemuck::pod_read_unaligned(&bytes[..size_of::<T>()])))
    }

    /// `readRawDataChunk`: advances within the cache or refills it; returns the chunk offset.
    fn next_chunk(&mut self, stream: &mut impl Read) -> Option<usize> {
        self.next_chunk_with(|buffer| {
            let filled = fill_from(stream, buffer);
            (filled.count, !filled.failed)
        })
    }

    fn next_chunk_with<F>(&mut self, mut read_block: F) -> Option<usize>
    where
        F: FnMut(&mut [u8]) -> (usize, bool),
    {
        match self
            .next_chunk_fallible(|buffer| Ok::<_, std::convert::Infallible>(read_block(buffer)))
        {
            Ok(start) => start,
            Err(never) => match never {},
        }
    }

    fn next_chunk_istream<R: Read>(
        &mut self,
        stream: &mut IstreamStream<R>,
    ) -> Result<Option<usize>, IstreamException> {
        self.next_chunk_fallible(|buffer| {
            stream
                .read_block(buffer)
                .map(|result| (result.gcount(), result.is_good()))
        })
    }

    fn next_chunk_fallible<F, E>(&mut self, mut read_block: F) -> Result<Option<usize>, E>
    where
        F: FnMut(&mut [u8]) -> Result<(usize, bool), E>,
    {
        let Some(position) = self.position else {
            return Ok(None);
        };
        let advanced = position + self.chunk_len;
        if advanced < self.end {
            self.position = Some(advanced);
            return Ok(Some(advanced));
        }
        self.position = None;
        self.refill_fallible(&mut read_block)
    }

    /// Reads whole cache chunks; a failed refill leaves data_read unchanged.
    fn refill_fallible<F, E>(&mut self, read_block: &mut F) -> Result<Option<usize>, E>
    where
        F: FnMut(&mut [u8]) -> Result<(usize, bool), E>,
    {
        let left = self.data_len - self.data_read;
        if left <= 0 {
            return Ok(None);
        }
        let to_read = if self.buffer_len as i64 > left {
            left as usize
        } else {
            self.buffer_len
        };
        let (count, succeeded) = read_block(&mut self.cache[..to_read])?;
        if !succeeded || count != to_read {
            return Ok(None);
        }
        self.position = Some(0);
        self.end = to_read;
        self.data_read += to_read as i64;
        Ok(Some(0))
    }

    fn check_stream_chunk_size<T: AnyBitPattern>(&self) -> Result<(), Failure> {
        let size = size_of::<T>();
        if size > self.chunk_len {
            return Err(exceptions::program_error(&format!(
                "Standard_ReadBuffer::ReadChunk: a {size}-byte value does not fit a {}-byte chunk",
                self.chunk_len
            )));
        }
        Ok(())
    }
}

/// Error from an exception-aware typed chunk read.
#[derive(Debug)]
pub enum ReadChunkIstreamError {
    /// The requested Rust value is larger than the native chunk.
    InvalidChunk(Failure),
    /// The native stream exception mask was triggered.
    Stream(IstreamException),
}

impl From<Failure> for ReadChunkIstreamError {
    fn from(error: Failure) -> Self {
        Self::InvalidChunk(error)
    }
}

impl From<IstreamException> for ReadChunkIstreamError {
    fn from(error: IstreamException) -> Self {
        Self::Stream(error)
    }
}

impl std::fmt::Display for ReadChunkIstreamError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidChunk(error) => std::fmt::Display::fmt(error, formatter),
            Self::Stream(error) => std::fmt::Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for ReadChunkIstreamError {}

/// The length `Init` keeps: whole chunks only (C++ `%` on `int64_t`, rounding toward zero)
/// unless the payload is partial; `truncated_length(13, 4, false) == 12`.
fn truncated_length(data_len: i64, chunk_len: usize, partial: bool) -> i64 {
    if partial {
        return data_len;
    }
    // int64_t(theChunkLen) wraps for sizes above i64::MAX, as the C++ conversion does.
    data_len.wrapping_sub(data_len.wrapping_rem(chunk_len as i64))
}
