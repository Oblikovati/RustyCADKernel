use super::IstreamStream;
use crate::standard::ReadLineBuffer;
use std::io::{self, Cursor, Read};

struct ErrorThenBytes {
    source: Cursor<Vec<u8>>,
    fail_once: bool,
}

impl Read for ErrorThenBytes {
    fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
        if self.fail_once {
            self.fail_once = false;
            return Err(io::Error::other("injected badbit"));
        }
        self.source.read(target)
    }
}

#[derive(Debug)]
struct InjectedSourceFault(&'static str);

impl std::fmt::Display for InjectedSourceFault {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for InjectedSourceFault {}

struct PayloadFailure;

impl Read for PayloadFailure {
    fn read(&mut self, _target: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other(InjectedSourceFault("native payload")))
    }
}

#[test]
fn badbit_stays_sticky_until_clear() {
    let source = ErrorThenBytes {
        source: Cursor::new(b"ok".to_vec()),
        fail_once: true,
    };
    let mut stream = IstreamStream::new(source);
    let mut lines = ReadLineBuffer::new(2);
    assert_read(&mut lines, &mut stream, false, 0);
    assert!(stream.is_bad() && stream.is_fail());
    assert_read(&mut lines, &mut stream, false, 0);
    stream.clear();
    assert_read(&mut lines, &mut stream, true, 2);
    assert_eq!(stream.into_inner().source.position(), 2);
}

#[test]
fn eof_and_fail_are_sticky_but_not_bad() {
    let mut stream = IstreamStream::new(Cursor::new(b"x".to_vec()));
    let mut lines = ReadLineBuffer::new(2);
    assert_read(&mut lines, &mut stream, true, 1);
    assert!(stream.is_eof() && stream.is_fail());
    assert!(!stream.is_bad());
    assert_read(&mut lines, &mut stream, true, 0);
    stream.clear();
    assert_read(&mut lines, &mut stream, true, 0);
    assert!(stream.is_eof() && stream.is_fail());
}

#[test]
fn manually_set_failbit_blocks_reads_without_setting_badbit() {
    let mut stream = IstreamStream::new(Cursor::new(b"ok".to_vec()));
    stream.set_status(false, true, false);
    let mut lines = ReadLineBuffer::new(2);
    assert_read(&mut lines, &mut stream, true, 0);
    assert!(stream.is_fail());
    assert!(!stream.is_bad());
    stream.clear();
    assert_read(&mut lines, &mut stream, true, 2);
}

#[test]
fn manually_set_badbit_also_reports_native_fail() {
    let mut stream = IstreamStream::new(Cursor::new(b"z".to_vec()));
    stream.set_status(false, false, true);
    assert!(stream.is_fail() && stream.is_bad());
    let mut byte = [0];
    assert!(stream.read(&mut byte).is_err());
}

#[test]
fn zero_length_read_does_not_set_eof_or_touch_the_source() {
    let mut stream = IstreamStream::new(Cursor::new(Vec::<u8>::new()));
    assert_eq!(stream.read(&mut []).unwrap(), 0);
    assert!(!stream.is_eof() && !stream.is_fail() && !stream.is_bad());
    let mut byte = [0];
    assert_eq!(stream.read(&mut byte).unwrap(), 0);
    assert!(stream.is_eof() && stream.is_fail());
}

#[test]
fn eofbit_only_sets_fail_on_the_next_nonzero_read() {
    let mut stream = IstreamStream::new(Cursor::new(b"z".to_vec()));
    stream.set_status(true, false, false);
    assert!(stream.is_eof());
    assert!(!stream.is_fail());
    let mut byte = [0];
    assert_eq!(stream.read(&mut byte).unwrap(), 0);
    assert!(stream.is_eof() && stream.is_fail());
    stream.clear();
    assert_eq!(stream.read(&mut byte).unwrap(), 1);
    assert_eq!(byte, [b'z']);
}

fn assert_read(
    lines: &mut ReadLineBuffer,
    stream: &mut impl Read,
    expected_good: bool,
    expected_count: usize,
) {
    let mut count = usize::MAX;
    assert_eq!(lines.read_stream(stream, 2, &mut count), expected_good);
    assert_eq!(count, expected_count);
}

#[test]
fn exception_masks_apply_after_state_mutation_and_distinguish_bits() {
    use crate::standard::IstreamExceptionMask;
    let mut stream = IstreamStream::new(Cursor::new(Vec::<u8>::new()));
    stream.set_status(false, false, true);
    stream
        .set_exceptions(IstreamExceptionMask::new(false, true, false))
        .unwrap();
    assert!(stream.is_bad() && stream.is_fail());
    assert!(
        stream
            .set_exceptions(IstreamExceptionMask::new(false, false, true))
            .is_err()
    );
    assert!(stream.clear_state(false, false, true).is_err());
    assert!(stream.is_bad());
    stream.clear();
    assert!(!stream.is_bad() && !stream.is_fail());
}

#[test]
fn badbit_masks_failbit_only_after_the_next_sentry() {
    use crate::standard::IstreamExceptionMask;
    let source = ErrorThenBytes {
        source: Cursor::new(Vec::new()),
        fail_once: true,
    };
    let mut stream = IstreamStream::new(source);
    stream
        .set_exceptions(IstreamExceptionMask::new(false, true, false))
        .unwrap();
    let mut byte = [0];
    let first = stream.read_block(&mut byte).unwrap();
    assert!(!first.is_good() && stream.is_bad());
    assert!(stream.read_block(&mut byte).unwrap_err().fail());
}

#[test]
fn zero_length_read_checks_existing_masked_state() {
    use crate::standard::IstreamExceptionMask;
    let mut stream = IstreamStream::new(Cursor::new(b"z".to_vec()));
    stream
        .set_exceptions(IstreamExceptionMask::new(false, false, true))
        .unwrap();
    stream.set_status(false, false, true);
    let error = stream.read_block(&mut []).unwrap_err();
    assert!(error.bad() && error.fail());
    assert!(stream.is_fail());
}

#[test]
fn masked_badbit_retains_the_source_error_payload() {
    use crate::standard::IstreamExceptionMask;
    let mut stream = IstreamStream::new(PayloadFailure);
    stream
        .set_exceptions(IstreamExceptionMask::new(false, false, true))
        .unwrap();
    let mut byte = [0];
    let error = stream.read_block(&mut byte).unwrap_err();
    let original = error
        .source_error()
        .and_then(std::io::Error::get_ref)
        .and_then(|source| source.downcast_ref::<InjectedSourceFault>());
    assert_eq!(original.map(|fault| fault.0), Some("native payload"));
    assert!(std::error::Error::source(&error).is_some());
}
