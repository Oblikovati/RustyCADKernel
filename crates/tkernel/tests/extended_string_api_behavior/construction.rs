//! Constructors, conversions to UTF-8, printing and hashing.
use super::{class, string, units};
use rustycad_tkernel::{
    ncollection::HashKey,
    tcollection::{AsciiString, ExtendedString},
};

#[test]
fn c_string_constructor_decodes_like_the_lenient_utf_iterator() {
    let decode = |bytes: &[u8]| ExtendedString::from_c_string(bytes, true);
    assert_eq!(
        decode("€😀".as_bytes()).as_utf16(),
        &[0x20AC, 0xD83D, 0xDE00]
    );
    // A lead byte followed by a plain byte still decodes: (0xC3 << 6) + 0x41 - 0x3080.
    assert_eq!(decode(b"\xC3A").as_utf16(), &[0x81]);
    assert_eq!(decode(b"\x80\xBF").as_utf16(), &[0x80, 0xBF]);
    // The overlong C0 80 decodes to zero and ends the string.
    assert_eq!(decode(b"A\xC0\x80B").as_utf16(), &[0x41]);
    // An encoded surrogate is dropped, not rejected.
    assert_eq!(decode(b"\xED\xA0\x80Z").as_utf16(), &[0x5A]);
}
#[test]
fn c_string_constructor_widens_what_is_not_utf8() {
    let widened = ExtendedString::from_c_string(b"\xF4\x90\x80\x80", true);
    assert_eq!(widened.as_utf16(), &[0xF4, 0x90, 0x80, 0x80]);
    // A sequence cut by the terminator is undefined in OCCT; the port widens the bytes.
    assert_eq!(
        ExtendedString::from_c_string(b"a\xE2\x82", true).as_utf16(),
        &[0x61, 0xE2, 0x82]
    );
    assert_eq!(
        ExtendedString::from_c_string(b"\xC3\xA9", false).as_utf16(),
        &[0xC3, 0xA9]
    );
    assert_eq!(
        ExtendedString::from_c_string(b"ab\0cd", false).as_utf16(),
        &units("ab")[..]
    );
    assert_eq!(ExtendedString::new("Hi").as_utf16(), &units("Hi")[..]);
    assert_eq!(ExtendedString::from_utf8("é").as_utf16(), &[0xE9]);
}
#[test]
fn ascii_constructor_keeps_embedded_nuls_only_when_widening() {
    let ascii = AsciiString::from_raw(b"A\0B");
    assert_eq!(
        ExtendedString::from_ascii_string(&ascii, false).as_utf16(),
        &[0x41, 0, 0x42]
    );
    assert_eq!(
        ExtendedString::from_ascii_string(&ascii, true).as_utf16(),
        &[0x41]
    );
    // Latin-1 "été" is read as one three-byte symbol: ((0xE9 << 6) + 0x74 << 6) + 0xE9 - 0xE2080.
    let latin = AsciiString::from_raw(b"\xE9t\xE9");
    assert_eq!(ExtendedString::from_ascii(&latin).as_utf16(), &[0x8D69]);
    let invalid = AsciiString::from_raw(b"\xF8AAAA");
    assert_eq!(
        ExtendedString::from_ascii(&invalid).as_utf16(),
        &[0xF8, 65, 65, 65, 65]
    );
}
#[test]
fn character_filler_and_number_constructors() {
    assert!(ExtendedString::from_char(0).is_empty());
    assert!(ExtendedString::from(0_u8).is_empty());
    assert_eq!(ExtendedString::from(233_u8).as_utf16(), &[233]);
    assert_eq!(ExtendedString::filled(3, 0).unwrap().as_utf16(), &[0, 0, 0]);
    assert_eq!(class(ExtendedString::filled(-1, 45)), "Standard_OutOfRange");
    assert_eq!(
        ExtendedString::from_integer(i32::MIN).to_utf8(),
        "-2147483648"
    );
    assert_eq!(ExtendedString::from_real(1e300).to_utf8(), "1e+300");
    assert_eq!(ExtendedString::from_real(f64::NAN).to_utf8(), "nan");
}
#[test]
fn wide_constructor_drops_values_utf16_cannot_hold() {
    let wide = [0x41, 0xD800, 0x11_0000, 0xFFFF_FFFF, 0x1F600, 0, 0x42];
    assert_eq!(
        ExtendedString::from_wide(&wide).as_utf16(),
        &[0x41, 0xD83D, 0xDE00]
    );
}
#[test]
fn span_constructor_checks_null_then_length() {
    assert_eq!(
        class(ExtendedString::from_span(None, -1)),
        "Standard_NullObject"
    );
    assert_eq!(
        class(ExtendedString::from_span(Some(&[65]), -1)),
        "Standard_OutOfRange"
    );
    let span = ExtendedString::from_span(Some(&[65, 0, 66]), 3).unwrap();
    assert_eq!(span.as_utf16(), &[65, 0, 66]);
    assert!(ExtendedString::from_span(Some(&[]), 0).unwrap().is_empty());
    assert_eq!(ExtendedString::from_utf16(&[65, 0, 66]).as_utf16(), &[65]);
    assert!(ExtendedString::empty_string().is_empty());
}
#[test]
#[should_panic(expected = "expected at least the 3 units")]
fn span_constructor_refuses_to_read_past_the_slice() {
    let _ = ExtendedString::from_span(Some(&[65]), 3);
}
#[test]
fn utf8_output_stops_at_zero_and_drops_lone_surrogates() {
    let text = ExtendedString::from_raw(&[0x41, 0xDC00, 0xD83D, 0xDE00, 0, 0x42]);
    assert_eq!(text.to_utf8(), "A😀");
    assert_eq!(text.utf8_length(), 5);
    let mut buffer = [0x7F_u8; 6];
    assert_eq!(text.write_utf8(&mut buffer).unwrap(), 5);
    assert_eq!(buffer[5], 0);
    assert_eq!(class(text.write_utf8(&mut [0; 5])), "Standard_OutOfRange");
    let mut printed = Vec::new();
    text.print(&mut printed).unwrap();
    assert_eq!(printed, "A😀".as_bytes());
    assert_eq!(text.to_string(), "A😀");
    assert_eq!(
        text.as_utf16_with_nul(),
        &[0x41, 0xDC00, 0xD83D, 0xDE00, 0, 0x42, 0]
    );
}
#[test]
#[cfg(target_pointer_width = "64")]
fn hash_uses_fnv_below_two_units_and_murmur_otherwise() {
    // Values printed by the native TCollection_ExtendedString::HashCode (64-bit Linux).
    let expected: [(&str, usize); 4] = [
        ("", 14_695_981_039_346_656_037),
        ("é", 17_841_711_379_143_803_052),
        ("a", 620_337_896_427_418_084),
        ("ab", 7_301_104_965_762_792_192),
    ];
    for (text, hash) in expected {
        assert_eq!(string(text).hash_code(), hash, "hash of {text:?}");
        assert_eq!(string(text).occt_hash(), hash);
    }
}
