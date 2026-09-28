//! Invalid Unicode and buffer boundaries must preserve the native conversion policy.
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
#[test]
fn utf16_conversion_discards_unpaired_surrogates() {
    let string = ExtendedString::from_utf16(&[65, 0xD800, 66, 0xDC00, 0xD83D, 0xDE00, 0]);
    assert_eq!(string.to_utf8(), "AB😀");
    assert_eq!(string.utf8_length(), 6);
    assert_eq!(
        AsciiString::from_extended(&string, None).as_bytes(),
        "AB😀".as_bytes()
    );
    assert!(string.write_utf8(&mut [0; 6]).is_err());
    let mut buffer = [0; 7];
    assert_eq!(string.write_utf8(&mut buffer).unwrap(), 6);
    assert_eq!(buffer[6], 0);
}
#[test]
fn utf16_padding_errors_and_missing_numeric_prefixes() {
    let mut string = ExtendedString::new("text");
    assert!(string.left_justify(-1, 32).is_err());
    assert!(string.right_justify(-1, 32).is_err());
    assert!(string.center(-1, 32).is_err());
    assert!(string.integer_value().is_err());
    assert!(string.real_value().is_err());
}
