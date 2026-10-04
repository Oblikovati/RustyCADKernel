use super::{FreadStream, ReadLineBuffer};
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

struct ScriptedFileSource {
    step: usize,
}

impl Read for ScriptedFileSource {
    fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
        let step = self.step;
        match step {
            0 => write_script_byte(self, target, b'a'),
            1 => fail_script_once(self),
            2..=5 => write_script_byte(self, target, b'b' + (step - 2) as u8),
            _ => Ok(0),
        }
    }
}

fn write_script_byte(
    source: &mut ScriptedFileSource,
    target: &mut [u8],
    byte: u8,
) -> io::Result<usize> {
    target[0] = byte;
    source.step += 1;
    Ok(1)
}

fn fail_script_once(source: &mut ScriptedFileSource) -> io::Result<usize> {
    source.step += 1;
    Err(io::Error::other("injected fread error"))
}

#[test]
fn file_hook_keeps_ferror_sticky_while_later_reads_fill_the_buffer() {
    let mut buffer = ReadLineBuffer::new(4);
    let mut stream = FreadStream::new(ScriptedFileSource { step: 0 });
    assert_file_read(&mut buffer, &mut stream, 1, b"a\0\0\0", false);
    assert_file_read(&mut buffer, &mut stream, 4, b"bcde", false);
    assert!(stream.is_error());
    assert_file_read(&mut buffer, &mut stream, 0, b"bcde", false);
    assert!(stream.is_eof());
    assert_file_read(&mut buffer, &mut stream, 0, b"bcde", false);
    stream.clear_status();
    assert_file_read(&mut buffer, &mut stream, 0, b"bcde", true);
    assert!(stream.is_eof());
}

fn assert_file_read(
    buffer: &mut ReadLineBuffer,
    stream: &mut FreadStream<ScriptedFileSource>,
    expected_count: usize,
    expected_buffer: &[u8],
    expected_success: bool,
) {
    let mut count = usize::MAX;
    let succeeded = buffer.read_stream_file(stream, 4, &mut count);
    assert_eq!((succeeded, count), (expected_success, expected_count));
    assert_eq!(&buffer.read_buffer[..], expected_buffer);
}

#[test]
fn file_hook_rejects_lengths_past_its_buffer_without_touching_stream_state() {
    let mut buffer = ReadLineBuffer::new(2);
    let mut stream = FreadStream::new(Cursor::new(b"ab".to_vec()));
    let mut count = usize::MAX;
    assert!(!buffer.read_stream_file(&mut stream, 3, &mut count));
    assert_eq!(count, 0);
    assert!(!stream.is_error());
    assert!(!stream.is_eof());
}

#[test]
fn file_line_methods_keep_the_native_last_block_count() {
    let mut buffer = ReadLineBuffer::new(4);
    let mut stream = FreadStream::new(Cursor::new(b"one\ntwo".to_vec()));
    assert_eq!(buffer.read_line_file(&mut stream), Some(b"one".as_slice()));
    let mut read_data = -1;
    assert_eq!(
        buffer.read_line_file_counted(&mut stream, &mut read_data),
        Some(b"two".as_slice())
    );
    assert_eq!(read_data, 0);
    assert!(stream.is_eof());
    assert!(buffer.read_line_file(&mut stream).is_none());
}

#[test]
fn file_line_error_keeps_read_data_zero_when_fread_fails() {
    let mut buffer = ReadLineBuffer::new(4);
    let mut stream = FreadStream::new(ScriptedFileSource { step: 0 });
    let mut read_data = -1;
    assert!(
        buffer
            .read_line_file_counted(&mut stream, &mut read_data)
            .is_none()
    );
    assert_eq!(read_data, 0);
    assert!(stream.is_error());
}

#[test]
fn file_stream_can_return_its_source() {
    let source = FreadStream::new(Cursor::new(b"abc".to_vec())).into_inner();
    assert_eq!(source.into_inner(), b"abc");
}
