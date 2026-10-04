use super::ReadBuffer;
use crate::standard::{IstreamExceptionMask, IstreamStream, ReadChunkIstreamError};
use std::io::{self, Read};

struct PartialFailure {
    prefix: &'static [u8],
    failed: bool,
}

impl Read for PartialFailure {
    fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
        if self.failed {
            return Ok(0);
        }
        self.failed = true;
        target[..self.prefix.len()].copy_from_slice(self.prefix);
        Err(io::Error::other("injected istream failure"))
    }
}

#[test]
fn exception_masked_cache_refill_preserves_native_partial_write_and_cursor() {
    let mut chunks = ReadBuffer::new(8, 4, false).unwrap();
    let source = PartialFailure {
        prefix: b"ab",
        failed: false,
    };
    let mut stream = IstreamStream::new(source);
    stream
        .set_exceptions(IstreamExceptionMask::new(false, false, true))
        .unwrap();
    let error = chunks.read_data_chunk_istream(&mut stream).unwrap_err();
    assert_eq!(error.gcount(), 0);
    assert!(error.bad() && error.fail());
    assert_eq!(chunks.data_read, 0);
    assert_eq!(chunks.position, None);
    assert_eq!(&chunks.cache[..4], b"ab\0\0");
}

#[test]
fn unmasked_stream_error_returns_no_chunk_after_partial_write() {
    let mut chunks = ReadBuffer::new(8, 4, false).unwrap();
    let source = PartialFailure {
        prefix: b"xy",
        failed: false,
    };
    let mut stream = IstreamStream::new(source);
    assert_eq!(chunks.read_data_chunk_istream(&mut stream).unwrap(), None);
    assert!(stream.is_bad());
    assert_eq!(chunks.data_read, 0);
    assert_eq!(&chunks.cache[..4], b"xy\0\0");
}

#[test]
fn typed_istream_chunk_distinguishes_value_size_from_stream_exception() {
    let mut chunks = ReadBuffer::new(8, 2, false).unwrap();
    let mut stream = IstreamStream::new(&b"abcd"[..]);
    let error = chunks
        .read_chunk_istream::<u32, _>(&mut stream)
        .unwrap_err();
    assert!(matches!(error, ReadChunkIstreamError::InvalidChunk(_)));

    let mut chunks = ReadBuffer::new(8, 4, false).unwrap();
    let source = PartialFailure {
        prefix: b"ab",
        failed: false,
    };
    let mut stream = IstreamStream::new(source);
    stream
        .set_exceptions(IstreamExceptionMask::new(false, false, true))
        .unwrap();
    let error = chunks
        .read_chunk_istream::<u32, _>(&mut stream)
        .unwrap_err();
    assert!(matches!(error, ReadChunkIstreamError::Stream(_)));
}
