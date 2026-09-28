//! Standard_ArrayStreamBuffer members added beyond the legacy suite.
use rustycad_tkernel::standard::ArrayStreamBuffer;
use std::io::SeekFrom;

#[test]
fn xsgetn_copies_what_is_left_and_advances() {
    let mut buffer = ArrayStreamBuffer::new(b"Hello");
    let mut out = [0; 3];
    assert_eq!(buffer.xsgetn(&mut out), 3);
    assert_eq!(&out, b"Hel");
    let mut rest = [0; 10];
    assert_eq!(buffer.xsgetn(&mut rest), 2);
    assert_eq!(&rest[..2], b"lo");
    assert_eq!(buffer.xsgetn(&mut rest), 0);
}

#[test]
fn uflow_reads_one_byte_until_the_end() {
    let mut buffer = ArrayStreamBuffer::new(b"ab");
    assert_eq!(buffer.uflow(), Some(b'a'));
    assert_eq!(buffer.uflow(), Some(b'b'));
    assert_eq!(buffer.uflow(), None);
    buffer.init(b"z");
    assert_eq!(buffer.uflow(), Some(b'z'));
}

#[test]
fn putting_back_a_high_byte_follows_the_char_signedness() {
    let mut buffer = ArrayStreamBuffer::new(&[0xFF, 0x41]);
    buffer.uflow();
    let expected = if std::ffi::c_char::MIN == 0 {
        Some(0xFF)
    } else {
        None
    };
    assert_eq!(buffer.put_back(Some(0xFF)), expected);
    let mut ascii = ArrayStreamBuffer::new(b"A");
    ascii.uflow();
    assert_eq!(ascii.put_back(Some(b'A')), Some(b'A'));
}

#[test]
fn seeks_that_would_leave_the_buffer_are_refused() {
    let mut buffer = ArrayStreamBuffer::new(b"abcd");
    buffer.seek_occt(SeekFrom::Start(2)).unwrap();
    assert!(buffer.seek_occt(SeekFrom::End(-1)).is_err());
    assert!(buffer.seek_occt(SeekFrom::Current(-3)).is_err());
    assert_eq!(buffer.available(), 2, "a refused seek keeps the position");
    assert_eq!(buffer.seek_occt(SeekFrom::Current(10)).unwrap(), 4);
    assert_eq!(buffer.seek_occt(SeekFrom::End(9)).unwrap(), 0);
}
