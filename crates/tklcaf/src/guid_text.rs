//! Compile-time GUID literals: the OCAF attributes define their identifiers as
//! `static Standard_GUID X("2a96b611-ec8b-11d0-bee7-080009dc3333")`; the port parses the
//! same text in a `const fn`, so a malformed literal is a compile error, never a runtime one.
use rustycad_tkernel::standard::{Guid, Uuid};

/// Parses the 36-character `8-4-4-4-12` hexadecimal text of a GUID at compile time;
/// `const ID: Guid = guid_from_text("2a96b611-ec8b-11d0-bee7-080009dc3333");`.
///
/// ```
/// use rustycad_tklcaf::guid_from_text;
/// const ID: rustycad_tkernel::standard::Guid = guid_from_text("2a96b611-ec8b-11d0-bee7-080009dc3333");
/// assert_eq!(ID.to_string(), "2a96b611-ec8b-11d0-bee7-080009dc3333");
/// ```
pub const fn guid_from_text(text: &str) -> Guid {
    let bytes = text.as_bytes();
    assert!(bytes.len() == 36, "GUID text must have 36 characters");
    let mut data4 = [0u8; 8];
    data4[0] = hex_value(bytes, 19, 2) as u8;
    data4[1] = hex_value(bytes, 21, 2) as u8;
    let mut index = 0;
    while index < 6 {
        data4[2 + index] = hex_value(bytes, 24 + 2 * index, 2) as u8;
        index += 1;
    }
    Guid::from_uuid(Uuid {
        data1: hex_value(bytes, 0, 8),
        data2: hex_value(bytes, 9, 4) as u16,
        data3: hex_value(bytes, 14, 4) as u16,
        data4,
    })
}

/// The value of `count` hexadecimal digits starting at `start`.
const fn hex_value(bytes: &[u8], start: usize, count: usize) -> u32 {
    let mut value: u32 = 0;
    let mut index = 0;
    while index < count {
        value = (value << 4) | hex_digit(bytes[start + index]);
        index += 1;
    }
    value
}

/// One hexadecimal digit; a compile error for anything else.
const fn hex_digit(byte: u8) -> u32 {
    match byte {
        b'0'..=b'9' => (byte - b'0') as u32,
        b'a'..=b'f' => (byte - b'a' + 10) as u32,
        b'A'..=b'F' => (byte - b'A' + 10) as u32,
        _ => panic!("GUID text holds a non-hexadecimal digit"),
    }
}
