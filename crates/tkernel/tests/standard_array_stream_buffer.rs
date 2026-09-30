//! The 20 unchanged-input legacy stream-buffer scenarios, using Rust I/O traits.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::ArrayStreamBuffer;
use std::io::{BufRead, Read, Seek, SeekFrom};

fn character(buffer: &mut ArrayStreamBuffer<'_>) -> Option<u8> {
    let mut byte = [0];
    (buffer.read(&mut byte).unwrap() == 1).then_some(byte[0])
}
fn word(buffer: &mut ArrayStreamBuffer<'_>) -> String {
    let mut result = Vec::new();
    while let Some(byte) = character(buffer) {
        if byte.is_ascii_whitespace() {
            break;
        }
        result.push(byte);
    }
    String::from_utf8(result).unwrap()
}
fn line(buffer: &mut ArrayStreamBuffer<'_>, delimiter: u8) -> String {
    let mut bytes = Vec::new();
    buffer.read_until(delimiter, &mut bytes).unwrap();
    if bytes.last() == Some(&delimiter) {
        bytes.pop();
    }
    String::from_utf8(bytes).unwrap()
}
#[test]
fn Standard_ArrayStreamBufferTest__ConstructorAndBasicUsage() {
    assert_eq!(
        word(&mut ArrayStreamBuffer::new(b"Hello, World!")),
        "Hello,"
    );
}
#[test]
fn Standard_ArrayStreamBufferTest__InitMethod() {
    let mut buffer = ArrayStreamBuffer::new(b"First data");
    assert_eq!(word(&mut buffer), "First");
    buffer.init(b"Second data");
    buffer.seek(SeekFrom::Start(0)).unwrap();
    assert_eq!(word(&mut buffer), "Second");
}
#[test]
fn Standard_ArrayStreamBufferTest__ReadFullContent() {
    assert_eq!(
        line(&mut ArrayStreamBuffer::new(b"Hello World"), b'\n'),
        "Hello World"
    );
}
#[test]
fn Standard_ArrayStreamBufferTest__CharacterByCharacterRead() {
    let mut buffer = ArrayStreamBuffer::new(b"ABC");
    assert_eq!(character(&mut buffer), Some(b'A'));
    assert_eq!(character(&mut buffer), Some(b'B'));
    assert_eq!(character(&mut buffer), Some(b'C'));
    // Successful reads are Rust's equivalent of istream.good(); a zero read is EOF.
    assert_eq!(character(&mut buffer), None);
}
#[test]
fn Standard_ArrayStreamBufferTest__SeekFromBeginning() {
    let mut buffer = ArrayStreamBuffer::new(b"0123456789");
    buffer.seek(SeekFrom::Start(5)).unwrap();
    assert_eq!(character(&mut buffer), Some(b'5'));
}
#[test]
fn Standard_ArrayStreamBufferTest__SeekFromEnd() {
    // Despite its name, the original test seeks from the beginning to position 7.
    let mut buffer = ArrayStreamBuffer::new(b"0123456789");
    buffer.seek(SeekFrom::Start(7)).unwrap();
    assert_eq!(character(&mut buffer), Some(b'7'));
}
#[test]
fn Standard_ArrayStreamBufferTest__SeekFromCurrent() {
    let mut buffer = ArrayStreamBuffer::new(b"0123456789");
    buffer.seek(SeekFrom::Start(2)).unwrap();
    buffer.seek(SeekFrom::Current(3)).unwrap();
    assert_eq!(character(&mut buffer), Some(b'5'));
}
#[test]
fn Standard_ArrayStreamBufferTest__TellPosition() {
    let mut buffer = ArrayStreamBuffer::new(b"0123456789");
    assert_eq!(buffer.stream_position().unwrap(), 0);
    buffer.seek(SeekFrom::Start(5)).unwrap();
    assert_eq!(buffer.stream_position().unwrap(), 5);
    character(&mut buffer);
    assert_eq!(buffer.stream_position().unwrap(), 6);
}
#[test]
fn Standard_ArrayStreamBufferTest__ReadBeyondBuffer() {
    let mut buffer = ArrayStreamBuffer::new(b"ABC");
    buffer.seek(SeekFrom::Start(10)).unwrap();
    assert_eq!(character(&mut buffer), None);
}
#[test]
fn Standard_ArrayStreamBufferTest__EmptyBuffer() {
    assert_eq!(character(&mut ArrayStreamBuffer::new(b"")), None);
}
#[test]
fn Standard_ArrayStreamBufferTest__NullPointerBuffer() {
    assert_eq!(character(&mut ArrayStreamBuffer::from_optional(None)), None);
}
#[test]
fn Standard_ArrayStreamBufferTest__BinaryData() {
    let mut buffer = ArrayStreamBuffer::new(&[1, 2, 3, 0, 4, 5]);
    for expected in [1, 2, 3, 0, 4, 5] {
        assert_eq!(character(&mut buffer), Some(expected));
    }
}
#[test]
fn Standard_ArrayStreamBufferTest__LargeBuffer() {
    let bytes = vec![b'X'; 10000];
    let mut buffer = ArrayStreamBuffer::new(&bytes);
    buffer.seek(SeekFrom::Start(5000)).unwrap();
    assert_eq!(character(&mut buffer), Some(b'X'));
    assert_eq!(buffer.stream_position().unwrap(), 5001);
}
#[test]
fn Standard_ArrayStreamBufferTest__MultipleReads() {
    let mut buffer = ArrayStreamBuffer::new(b"Line1\nLine2\nLine3");
    assert_eq!(line(&mut buffer, b'\n'), "Line1");
    assert_eq!(line(&mut buffer, b'\n'), "Line2");
    assert_eq!(line(&mut buffer, b'\n'), "Line3");
}
#[test]
fn Standard_ArrayStreamBufferTest__SeekToSpecificPosition() {
    let mut buffer = ArrayStreamBuffer::new(b"0123456789");
    buffer.seek(SeekFrom::Start(7)).unwrap();
    assert_eq!(character(&mut buffer), Some(b'7'));
    assert_eq!(buffer.stream_position().unwrap(), 8);
}
#[test]
fn Standard_ArrayStreamBufferTest__ReadAfterSeekToEnd() {
    let mut buffer = ArrayStreamBuffer::new(b"Hello");
    buffer.seek(SeekFrom::End(0)).unwrap();
    assert_eq!(character(&mut buffer), None);
}
#[test]
fn Standard_ArrayStreamBufferTest__BackwardSeek() {
    let mut buffer = ArrayStreamBuffer::new(b"0123456789");
    buffer.seek(SeekFrom::End(0)).unwrap();
    buffer.seek(SeekFrom::Current(-5)).unwrap();
    assert_eq!(character(&mut buffer), Some(b'5'));
}
#[test]
fn Standard_ArrayStreamBufferTest__MultilineContent() {
    let text = "First line\nSecond line\nThird line";
    assert_eq!(line(&mut ArrayStreamBuffer::new(text.as_bytes()), 0), text);
}
#[test]
fn Standard_ArrayStreamBufferTest__ReadBlock() {
    let mut buffer = ArrayStreamBuffer::new(b"ABCDEFGHIJ");
    let mut block = [0; 5];
    let count = buffer.read(&mut block[..4]).unwrap();
    assert_eq!(&block[..4], b"ABCD");
    assert_eq!(block[4], 0);
    assert_eq!(count, 4);
}
#[test]
fn Standard_ArrayStreamBufferTest__AvailableCharacters() {
    let mut buffer = ArrayStreamBuffer::new(b"Testing");
    assert_eq!(buffer.available(), 7);
    character(&mut buffer);
    assert_eq!(buffer.available(), 6);
}
