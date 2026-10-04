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
