use super::ReadLineBuffer;
use crate::standard::{IstreamExceptionMask, IstreamStream};
use std::io::{self, Cursor, Read};

struct WriteThenError {
    prefix: &'static [u8],
    failed: bool,
}

impl Read for WriteThenError {
    fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
        if !self.failed {
            self.failed = true;
            target[..self.prefix.len()].copy_from_slice(self.prefix);
            return Err(io::Error::other("injected istream badbit"));
        }
        Ok(0)
    }
}

#[test]
fn exception_mask_read_retains_native_count_and_partial_buffer_order() {
    let source = WriteThenError {
        prefix: b"ab",
        failed: false,
    };
    let mut stream = IstreamStream::new(source);
    stream
        .set_exceptions(IstreamExceptionMask::new(false, false, true))
        .unwrap();
    let mut lines = ReadLineBuffer::new(4);
    let mut count = 77;
    let error = lines
        .read_stream_istream(&mut stream, 4, &mut count)
        .unwrap_err();
    assert_eq!(
        (error.gcount(), error.eof(), error.fail(), error.bad()),
        (0, false, true, true)
    );
    assert_eq!(count, 77);
    assert_eq!(&lines.read_buffer[..], b"ab\0\0");
}

#[test]
fn exception_mask_reuses_native_line_state_when_fetch_throws() {
    let mut lines = ReadLineBuffer::new(4);
    lines.read_buffer.copy_from_slice(b"prev");
    lines.position = 4;
    lines.last_read = 4;
    let mut length = usize::MAX;
    let mut read_data = i64::MAX;
    let source = WriteThenError {
        prefix: b"xy",
        failed: false,
    };
    let mut failing = IstreamStream::new(source);
    failing
        .set_exceptions(IstreamExceptionMask::new(false, false, true))
        .unwrap();
    let error = lines
        .read_line_istream(&mut failing, &mut length, &mut read_data)
        .unwrap_err();
    assert!(error.bad() && error.fail());
    assert_eq!(
        (length, read_data, lines.position, lines.last_read),
        (0, 0, 4, 4)
    );
    assert_eq!(&lines.read_buffer[..], b"xyev");
}

#[test]
fn exception_mask_propagates_through_the_unaccounted_line_overload() {
    let mut lines = ReadLineBuffer::new(4);
    lines.read_buffer.copy_from_slice(b"prev");
    lines.position = 4;
    lines.last_read = 4;
    let source = WriteThenError {
        prefix: b"xy",
        failed: false,
    };
    let mut stream = IstreamStream::new(source);
    stream
        .set_exceptions(IstreamExceptionMask::new(false, false, true))
        .unwrap();
    let mut length = usize::MAX;
    let error = lines
        .read_line_istream_unaccounted(&mut stream, &mut length)
        .unwrap_err();
    assert!(error.bad());
    assert_eq!((length, lines.position, lines.last_read), (0, 4, 4));
    assert_eq!(&lines.read_buffer[..], b"xyev");
}

#[test]
fn unmasked_badbit_keeps_zero_gcount_after_partial_buffer_write() {
    let source = WriteThenError {
        prefix: b"ab",
        failed: false,
    };
    let mut stream = IstreamStream::new(source);
    let mut lines = ReadLineBuffer::new(4);
    let mut count = usize::MAX;
    assert!(
        !lines
            .read_stream_istream(&mut stream, 4, &mut count)
            .unwrap()
    );
    assert_eq!(count, 0);
    assert!(stream.is_bad());
    assert_eq!(&lines.read_buffer[..], b"ab\0\0");
}

#[test]
fn setting_or_clearing_a_masked_state_mutates_before_returning_error() {
    let mut stream = IstreamStream::new(Cursor::new(Vec::<u8>::new()));
    stream.set_status(false, true, false);
    let error = stream
        .set_exceptions(IstreamExceptionMask::new(false, true, false))
        .unwrap_err();
    assert!(error.fail() && !error.bad());
    assert!(stream.clear_state(false, true, false).is_err());
    assert!(stream.is_fail());
    stream.clear();
    assert!(!stream.is_fail());
}

#[test]
fn short_read_reports_partial_gcount_before_masked_eof_failure() {
    let mut stream = IstreamStream::new(Cursor::new(b"xy".to_vec()));
    stream
        .set_exceptions(IstreamExceptionMask::new(false, true, false))
        .unwrap();
    let mut buffer = [0; 4];
    let error = stream.read_block(&mut buffer).unwrap_err();
    assert_eq!(error.gcount(), 2);
    assert!(error.eof() && error.fail() && !error.bad());
    assert_eq!(&buffer[..2], b"xy");
}
