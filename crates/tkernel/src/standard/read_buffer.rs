// Rust translation of FoundationClasses/TKernel/Standard/Standard_ReadBuffer.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_ReadBuffer`: reads a stream in fixed-size chunks through a 4096-byte cache.
use super::stream_fill::fill_from;
use super::{Failure, exceptions};
use bytemuck::AnyBitPattern;
use std::io::Read;
mod file_chunks;

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
        let size = size_of::<T>();
        if size > self.chunk_len {
            return Err(exceptions::program_error(&format!(
                "Standard_ReadBuffer::ReadChunk: a {size}-byte value does not fit a {}-byte chunk",
                self.chunk_len
            )));
        }
        let chunk = self.read_data_chunk(stream);
        Ok(chunk.map(|bytes| bytemuck::pod_read_unaligned(&bytes[..size])))
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
        let advanced = self.position? + self.chunk_len;
        if advanced < self.end {
            self.position = Some(advanced);
            return Some(advanced);
        }
        self.position = None;
        self.refill(&mut read_block)
    }

    /// Reads as many whole chunks as the cache holds, or the data left; `None` at the end
    /// of data or after a short read.
    fn refill<F>(&mut self, read_block: &mut F) -> Option<usize>
    where
        F: FnMut(&mut [u8]) -> (usize, bool),
    {
        let left = self.data_len - self.data_read;
        if left <= 0 {
            return None;
        }
        let to_read = if self.buffer_len as i64 > left {
            left as usize
        } else {
            self.buffer_len
        };
        let (count, succeeded) = read_block(&mut self.cache[..to_read]);
        if !succeeded || count != to_read {
            return None;
        }
        self.position = Some(0);
        self.end = to_read;
        self.data_read += to_read as i64;
        Some(0)
    }
}

/// The length `Init` keeps: whole chunks only (C++ `%` on `int64_t`, rounding toward zero)
/// unless the payload is partial; `truncated_length(13, 4, false) == 12`.
fn truncated_length(data_len: i64, chunk_len: usize, partial: bool) -> i64 {
    if partial {
        return data_len;
    }
    // int64_t(theChunkLen) wraps for sizes above i64::MAX, as the C++ conversion does.
    data_len.wrapping_sub(data_len.wrapping_rem(chunk_len as i64))
}
