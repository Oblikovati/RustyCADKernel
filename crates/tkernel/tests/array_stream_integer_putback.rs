//! Supplemental protected-hook coverage; no additional OCCT legacy declarations.
use rustycad_tkernel::standard::ArrayStreamBuffer;

#[test]
fn signed_character_request_restores_high_byte() {
    for byte in [0, 65, 127, 128, 129, 254, 255] {
        let storage = [byte];
        let mut buffer = ArrayStreamBuffer::new(&storage);
        assert_eq!(buffer.uflow(), Some(byte));
        let promoted = i32::from(byte as std::ffi::c_char);
        assert_eq!(buffer.put_back_integer(promoted), Some(byte));
        assert_eq!(buffer.available(), 1);
        assert_eq!(buffer.put_back_integer(-1), None);
        assert_eq!(buffer.peek(), Some(byte));
    }
}

#[test]
fn nonmatching_integer_preserves_position_and_eof_restores_byte() {
    let mut buffer = ArrayStreamBuffer::new(&[128]);
    assert_eq!(buffer.uflow(), Some(128));
    for requested in [i32::MIN, -129, 256, i32::MAX] {
        assert_eq!(buffer.put_back_integer(requested), None);
        assert_eq!(buffer.available(), 0);
    }
    assert_eq!(buffer.put_back_integer(-1), Some(128));
    assert_eq!(buffer.available(), 1);
}

#[test]
fn byte_wrapper_preserves_native_promoted_character_comparison() {
    let mut buffer = ArrayStreamBuffer::new(&[128]);
    assert_eq!(buffer.uflow(), Some(128));
    let expected = (std::ffi::c_char::MIN == 0).then_some(128);
    assert_eq!(buffer.put_back(Some(128)), expected);
    if expected.is_none() {
        assert_eq!(buffer.put_back(None), Some(128));
    }
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn integer_hook_matches_native_linux_receipt() {
    let receipt = include_str!("../../../tools/oracle/array_stream_integer_putback.expected");
    for row in receipt.lines() {
        let fields: Vec<i32> = row
            .split_whitespace()
            .map(|word| word.parse().unwrap())
            .collect();
        let storage = [fields[0] as u8];
        let mut buffer = ArrayStreamBuffer::new(&storage);
        assert_eq!(buffer.uflow().map_or(-1, i32::from), fields[2]);
        assert_eq!(
            buffer.put_back_integer(fields[1]).map_or(-1, i32::from),
            fields[3]
        );
        assert_eq!(buffer.available(), fields[4] as usize);
        assert_eq!(buffer.put_back_integer(-1).map_or(-1, i32::from), fields[5]);
    }
}
