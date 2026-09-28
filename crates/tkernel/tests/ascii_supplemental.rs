//! Additional exact-oracle discoveries beyond the source declaration assertions.
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
#[test]
fn view_constructor_and_copy_have_different_embedded_nul_behavior() {
    let constructed = AsciiString::from_view(b"a\0b");
    assert_eq!(constructed.len(), 1);
    let mut copied = AsciiString::default();
    copied.copy(&b"a\0b"[..]);
    assert_eq!(copied.len(), 3);
    assert_eq!(copied.useful_length(), 1);
    copied.copy(&b"3.3\0junk"[..]);
    assert!(copied.is_real_value(true));
    assert_eq!(copied.real_value().unwrap(), 3.3);
}
#[test]
fn real_general_format_and_numeric_errors_match_occt() {
    for (value, expected) in [
        (1e-5, "1e-05"),
        (1e6, "1e+06"),
        (-0.0, "-0"),
        (999999.5, "1e+06"),
    ] {
        assert_eq!(
            AsciiString::from_real(value).as_bytes(),
            expected.as_bytes()
        );
    }
    assert!(AsciiString::new("x").integer_value().is_err());
    assert!(AsciiString::new("x").real_value().is_err());
    assert!(AsciiString::new("").is_real_value(true));
    assert_eq!(
        AsciiString::new("2147483648").integer_value().unwrap(),
        i32::MIN
    );
}
#[test]
fn extended_replacement_keeps_latin1_bytes() {
    let extended = ExtendedString::from_utf16(&[0xE9, 0x20AC, 0]);
    assert!(!extended.is_empty());
    assert_eq!(
        AsciiString::from_extended(&extended, Some(b'?')).as_bytes(),
        &[0xE9, b'?']
    );
    assert_eq!(
        AsciiString::from_extended(&extended, Some(0)).as_bytes(),
        "é€".as_bytes()
    );
}
