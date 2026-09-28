//! Standard_ReadBuffer and Standard_ReadLineBuffer.
use rustycad_tkernel::standard::{ReadBuffer, ReadLineBuffer};
use std::io::{self, Cursor, Read};

/// A stream that serves its bytes and then fails, like a device error mid-file.
struct BrokenStream {
    bytes: Cursor<Vec<u8>>,
}
impl Read for BrokenStream {
    fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
        match self.bytes.read(destination)? {
            0 => Err(io::Error::other("device error")),
            count => Ok(count),
        }
    }
}

fn lines(buffer: &mut ReadLineBuffer, text: &[u8]) -> Vec<Vec<u8>> {
    let mut stream = Cursor::new(text.to_vec());
    let mut lines = Vec::new();
    while let Some(line) = buffer.read_line(&mut stream) {
        lines.push(line.to_vec());
    }
    lines
}

#[test]
fn chunks_are_truncated_to_whole_chunks_unless_partial() {
    let data: Vec<u8> = (0..13).collect();
    let mut whole = ReadBuffer::new(13, 4, false).unwrap();
    let mut stream = Cursor::new(data.clone());
    let mut chunks = Vec::new();
    while let Some(chunk) = whole.read_data_chunk(&mut stream) {
        chunks.push(chunk.to_vec());
    }
    assert_eq!(
        chunks,
        [vec![0, 1, 2, 3], vec![4, 5, 6, 7], vec![8, 9, 10, 11]]
    );
    assert!(whole.is_done());
    let mut partial = ReadBuffer::new(13, 4, true).unwrap();
    let mut stream = Cursor::new(data);
    let last = (0..4)
        .filter_map(|_| partial.read_data_chunk(&mut stream).map(|c| c.to_vec()))
        .last();
    assert_eq!(
        last,
        Some(vec![12, 0, 0, 0]),
        "the tail keeps the cache's old bytes"
    );
    assert!(partial.is_done());
}

#[test]
fn a_short_stream_ends_reading_for_good() {
    let mut buffer = ReadBuffer::new(8, 4, false).unwrap();
    let mut stream = Cursor::new(vec![1, 2, 3]);
    assert!(buffer.read_data_chunk(&mut stream).is_none());
    assert!(
        buffer
            .read_data_chunk(&mut Cursor::new(vec![9; 8]))
            .is_none()
    );
    assert!(!buffer.is_done());
}

#[test]
fn negative_lengths_round_toward_zero_and_never_finish() {
    let mut buffer = ReadBuffer::new(-5, 4, false).unwrap();
    assert!(!buffer.is_done(), "-5 - (-5 % 4) == -4, never reached");
    assert!(
        buffer
            .read_data_chunk(&mut Cursor::new(vec![0; 8]))
            .is_none()
    );
    assert!(ReadBuffer::new(0, 4, false).unwrap().is_done());
}

#[test]
fn oversized_or_empty_chunks_are_refused() {
    let failure = ReadBuffer::new(8, 4097, false).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert_eq!(
        failure.message(),
        "Internal error - chunk size is greater then preallocated buffer"
    );
    assert!(ReadBuffer::new(8, 0, false).is_err());
    let mut buffer = ReadBuffer::new(8, 4, false).unwrap();
    assert!(buffer.init(8, 5000, false).is_err());
    assert!(
        buffer
            .read_data_chunk(&mut Cursor::new(vec![0; 8]))
            .is_none()
    );
}

#[test]
fn typed_chunks_are_copied_out_and_must_fit() {
    let values: Vec<u8> = [7u32, 9].iter().flat_map(|v| v.to_ne_bytes()).collect();
    let mut buffer = ReadBuffer::new(8, 4, false).unwrap();
    let mut stream = Cursor::new(values);
    assert_eq!(buffer.read_chunk::<u32>(&mut stream).unwrap(), Some(7));
    assert_eq!(buffer.read_chunk::<u32>(&mut stream).unwrap(), Some(9));
    assert_eq!(buffer.read_chunk::<u32>(&mut stream).unwrap(), None);
    assert!(buffer.read_chunk::<u64>(&mut stream).is_err());
}

#[test]
fn a_cache_refill_spans_many_chunks() {
    let data: Vec<u8> = (0..10_000u32).map(|i| (i % 251) as u8).collect();
    let mut buffer = ReadBuffer::new(10_000, 1000, false).unwrap();
    let mut stream = Cursor::new(data.clone());
    let mut joined = Vec::new();
    while let Some(chunk) = buffer.read_data_chunk(&mut stream) {
        joined.extend_from_slice(chunk);
    }
    assert_eq!(joined, data);
}

#[test]
fn lines_drop_their_terminators_across_buffer_boundaries() {
    for size in [1, 2, 3, 7, 4096] {
        let mut buffer = ReadLineBuffer::new(size);
        let found = lines(&mut buffer, b"first\r\nsecond\n\nlast");
        assert_eq!(
            found,
            [&b"first"[..], b"second", b"", b"last"],
            "buffer {size}"
        );
    }
    let mut lone = ReadLineBuffer::new(4);
    assert_eq!(lines(&mut lone, b"a\rb\n"), [b"a\rb"]);
}

#[test]
fn multiline_mode_joins_escaped_newlines() {
    let mut gap = ReadLineBuffer::new(4096);
    gap.set_multiline_mode(true, true);
    assert!(gap.is_multiline_mode() && gap.to_put_gap_in_multiline());
    assert_eq!(lines(&mut gap, b"1/2/3\\\n4/5/6\n"), [b"1/2/3 4/5/6"]);
    let mut tight = ReadLineBuffer::new(4096);
    tight.set_multiline_mode(true, false);
    assert_eq!(lines(&mut tight, b"1/2/3\\\r\n4/5/6\n"), [b"1/2/34/5/6"]);
}

#[test]
fn a_backslash_at_a_block_end_can_swallow_later_newlines() {
    // Native quirk: "a\b" split after the backslash marks a continuation, so the next
    // newlines do not end the line and the rest of the stream becomes one line.
    let mut buffer = ReadLineBuffer::new(2);
    buffer.set_multiline_mode(true, true);
    assert_eq!(lines(&mut buffer, b"a\\b\nc\nd\n"), [b"a b\nc\nd\n"]);
}

#[test]
fn an_empty_carry_over_that_never_held_bytes_ends_the_stream() {
    let mut buffer = ReadLineBuffer::new(2);
    buffer.set_multiline_mode(true, false);
    assert!(lines(&mut buffer, b"\\\n\\\n").is_empty());
}

#[test]
fn read_counts_report_the_last_block_of_the_call() {
    let mut buffer = ReadLineBuffer::new(4);
    let mut stream = Cursor::new(b"abcdef\ngh\n".to_vec());
    let mut read = 0;
    // Blocks are "abcd", "ef\ng" and "h\n"; only the last block of each call counts.
    assert_eq!(
        buffer.read_line_counted(&mut stream, &mut read),
        Some(&b"abcdef"[..])
    );
    assert_eq!(read, 4);
    assert_eq!(
        buffer.read_line_counted(&mut stream, &mut read),
        Some(&b"gh"[..])
    );
    assert_eq!(read, 2);
    assert_eq!(buffer.read_line_counted(&mut stream, &mut read), None);
    assert_eq!(read, 0);
}

#[test]
fn clear_resets_modes_and_drops_buffered_text() {
    let mut buffer = ReadLineBuffer::new(4096);
    buffer.set_multiline_mode(true, false);
    let mut stream = Cursor::new(b"one\ntwo\nthree\n".to_vec());
    assert_eq!(buffer.read_line(&mut stream), Some(&b"one"[..]));
    buffer.clear();
    assert!(!buffer.is_multiline_mode() && buffer.to_put_gap_in_multiline());
    assert_eq!(
        buffer.read_line(&mut stream),
        None,
        "the buffered lines were dropped"
    );
}

#[test]
fn a_failing_stream_returns_no_line() {
    let mut buffer = ReadLineBuffer::new(4);
    let mut stream = BrokenStream {
        bytes: Cursor::new(b"ab".to_vec()),
    };
    assert_eq!(buffer.read_line(&mut stream), None);
    assert!(
        ReadBuffer::new(4, 4, false)
            .unwrap()
            .read_data_chunk(&mut BrokenStream {
                bytes: Cursor::new(b"ab".to_vec())
            })
            .is_none()
    );
}
