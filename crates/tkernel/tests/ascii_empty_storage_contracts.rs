//! Storage identity contracts observed in pinned OCCT's TCollection_AsciiString.
use rustycad_tkernel::tcollection::AsciiString;

fn empty_buffer_address(text: &AsciiString) -> *const u8 {
    assert_eq!(text.as_bytes_with_nul(), &[0]);
    text.as_bytes_with_nul().as_ptr()
}

#[test]
fn empty_constructors_share_the_native_default_buffer() {
    let expected = empty_buffer_address(AsciiString::empty_string());
    for text in [
        AsciiString::default(),
        AsciiString::new(""),
        AsciiString::from_raw(&[]),
        AsciiString::from_char(0),
        AsciiString::filled(0, b'x'),
        AsciiString::default().clone(),
    ] {
        assert_eq!(empty_buffer_address(&text), expected);
    }
}

#[test]
fn clear_copy_and_move_source_reset_to_shared_storage() {
    let expected = empty_buffer_address(AsciiString::empty_string());
    let mut text = AsciiString::new("owned");
    text.clear();
    assert_eq!(empty_buffer_address(&text), expected);
    text.copy("owned");
    text.copy("");
    assert_eq!(empty_buffer_address(&text), expected);
    text.copy("owned");
    let previous = text.as_bytes_with_nul().as_ptr();
    let moved = AsciiString::move_from(&mut text);
    assert_eq!(moved.as_bytes_with_nul().as_ptr(), previous);
    assert_eq!(empty_buffer_address(&text), expected);
}

#[test]
fn truncation_retains_owned_storage_but_cloning_empty_does_not() {
    let mut text = AsciiString::new("owned");
    let previous = text.as_bytes_with_nul().as_ptr();
    text.truncate(0).unwrap();
    assert_eq!(empty_buffer_address(&text), previous);
    assert_eq!(text, AsciiString::default());
    assert_eq!(
        empty_buffer_address(&text.clone()),
        empty_buffer_address(AsciiString::empty_string())
    );
}

#[test]
fn shared_empty_case_adjustment_preserves_storage() {
    let mut text = AsciiString::default();
    text.capitalize();
    text.right_adjust();
    text.truncate(0).unwrap();
    assert_eq!(
        empty_buffer_address(&text),
        empty_buffer_address(AsciiString::empty_string())
    );
}

#[test]
fn shared_empty_insertion_and_padding_preserve_termination() {
    let mut text = AsciiString::default();
    text.insert_char(1, b'a').unwrap();
    assert_eq!(text.as_bytes_with_nul(), b"a\0");
    text.clear();
    text.left_justify(2, b'.').unwrap();
    assert_eq!(text.as_bytes_with_nul(), b"..\0");
    text.clear();
    text.right_justify(2, b'!').unwrap();
    assert_eq!(text.as_bytes_with_nul(), b"!!\0");
}

#[test]
fn removing_all_bytes_retains_native_storage_identity() {
    let mut text = AsciiString::new("aAa");
    let previous = text.as_bytes_with_nul().as_ptr();
    text.remove_all_with_case(b'a', false);
    assert_eq!(empty_buffer_address(&text), previous);
    let mut shared = AsciiString::default();
    shared.remove_all(0);
    assert_eq!(
        empty_buffer_address(&shared),
        empty_buffer_address(AsciiString::empty_string())
    );
}

#[test]
fn removing_identical_bytes_keeps_the_owned_terminator() {
    let mut text = AsciiString::new("aaa");
    let previous = text.as_bytes_with_nul().as_ptr();
    text.remove_all(b'a');
    assert_eq!(empty_buffer_address(&text), previous);
}

#[test]
fn clone_from_reuses_shorter_storage_and_resets_empty_source() {
    let mut destination = AsciiString::new("longer");
    let previous = destination.as_bytes_with_nul().as_ptr();
    destination.clone_from(&AsciiString::new("x"));
    assert_eq!(destination.as_bytes_with_nul().as_ptr(), previous);
    assert_eq!(destination.as_bytes_with_nul(), b"x\0");
    destination.clone_from(&AsciiString::default());
    assert_eq!(
        empty_buffer_address(&destination),
        empty_buffer_address(AsciiString::empty_string())
    );
}
