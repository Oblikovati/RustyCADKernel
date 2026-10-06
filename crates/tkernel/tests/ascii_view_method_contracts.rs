//! Native string-view construction scans for NUL while view operations preserve their full span.
use rustycad_tkernel::tcollection::AsciiString;

#[test]
fn view_and_bounded_constructors_scan_the_first_nul() {
    for (view, expected) in [(&b"a\0b"[..], &b"a"[..]), (&b"\0b"[..], &b""[..])] {
        assert_eq!(AsciiString::from_view(view).as_bytes(), expected);
        assert_eq!(AsciiString::with_max_length(view, 3).as_bytes(), expected);
    }
}

#[test]
fn view_copy_and_explicit_span_preserve_embedded_nuls() {
    let view = &b"a\0b"[..];
    let mut copied = AsciiString::new("before");
    copied.copy(view);
    assert_eq!(copied.as_bytes(), view);
    assert_eq!(AsciiString::from_raw(view).as_bytes(), view);
}

#[test]
fn view_append_cat_and_operators_preserve_embedded_nuls() {
    let view = &b"a\0b"[..];
    let mut appended = AsciiString::new("X");
    appended.assign_cat(view);
    assert_eq!(appended.as_bytes(), b"Xa\0b");
    assert_eq!(AsciiString::new("X").cat(view).as_bytes(), b"Xa\0b");
    assert_eq!((&AsciiString::new("X") + view).as_bytes(), b"Xa\0b");
    let mut plus_assigned = AsciiString::new("X");
    plus_assigned += view;
    assert_eq!(plus_assigned.as_bytes(), b"Xa\0b");
}
