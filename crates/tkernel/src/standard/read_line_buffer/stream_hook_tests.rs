use super::ReadLineBuffer;
use std::io::{self, Cursor, Read};

struct FailingStream {
    bytes: Cursor<Vec<u8>>,
}

impl Read for FailingStream {
    fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
        let count = self.bytes.read(target)?;
        if count == 0 {
            return Err(io::Error::other("injected stream failure"));
        }
        Ok(count)
    }
}

#[test]
fn stream_hook_reports_full_then_short_then_eof_reads() {
    let mut buffer = ReadLineBuffer::new(4);
    let mut stream = Cursor::new(b"abcdef".to_vec());
    assert_read(&mut buffer, &mut stream, 4, true, b"abcd");
    assert_read(&mut buffer, &mut stream, 2, true, b"efcd");
    assert_read(&mut buffer, &mut stream, 0, true, b"efcd");
}

fn assert_read(
    buffer: &mut ReadLineBuffer,
    stream: &mut impl Read,
    expected_count: usize,
    expected_success: bool,
    expected_buffer: &[u8],
) {
    let mut count = usize::MAX;
    let succeeded = buffer.read_stream(stream, 4, &mut count);
    assert_eq!(succeeded, expected_success);
    assert_eq!(count, expected_count);
    assert_eq!(&buffer.read_buffer[..], expected_buffer);
}

#[test]
fn stream_hook_preserves_count_and_buffer_when_stream_is_already_bad() {
    let mut buffer = ReadLineBuffer::new(4);
    let mut initial = Cursor::new(b"old!".to_vec());
    assert_read(&mut buffer, &mut initial, 4, true, b"old!");
    let mut stream = FailingStream {
        bytes: Cursor::new(Vec::new()),
    };
    assert_read(&mut buffer, &mut stream, 0, false, b"old!");
}

#[test]
fn stream_hook_rejects_a_length_outside_the_allocated_buffer() {
    let mut buffer = ReadLineBuffer::new(2);
    let mut stream = Cursor::new(b"abc".to_vec());
    let mut count = usize::MAX;
    assert!(!buffer.read_stream(&mut stream, 3, &mut count));
    assert_eq!(count, 0);
    assert_eq!(stream.position(), 0);
}
