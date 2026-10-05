//! Native-backed checks for the literal and string-view constructor surface.
use rustycad_tkernel::tcollection::AsciiString;

#[test]
fn char_array_constructor_truncates_but_string_view_surface_keeps_full_span() {
    let literal = *b"ab\0cd\0";
    let truncated = AsciiString::from(&literal);
    assert_eq!(truncated.as_bytes(), b"ab");
    let view = AsciiString::from_view(b"a\0b");
    assert_eq!(view.as_bytes(), b"a");
    let embedded = AsciiString::from_raw(b"A\0z");
    let lower = b"A\0a";
    assert!(!embedded.is_less(lower));
    assert!(embedded.is_greater(lower));
    assert_eq!(embedded.as_bytes(), b"A\0z");
}
