//! Native UtfString ownership and raw-buffer method contracts.
use rustycad_tkernel::ncollection::{UtfString, WideChar};

#[test]
fn raw_buffer_and_symbol_pointers_identify_live_encoding_units() {
    let text = UtfString::<u8>::from_unicode_units("Aé😀".as_bytes(), -1);
    assert_eq!(text.to_c_ptr(), text.to_c_string().as_ptr());
    assert_eq!(text.get_char_buffer_ptr(0), text.to_c_ptr());
    assert_eq!(text.get_char_buffer_ptr(1), text.to_c_ptr().wrapping_add(1));
    assert_eq!(text.get_char_buffer_ptr(2), text.to_c_ptr().wrapping_add(3));
    assert!(text.get_char_buffer_ptr(-1).is_null());
    assert!(text.get_char_buffer_ptr(3).is_null());
    let empty = UtfString::<u32>::new();
    assert!(!empty.to_c_ptr().is_null());
    assert_eq!(empty.to_c_string(), [0]);
}

#[test]
fn move_construction_leaves_native_null_source_then_clear_restores_terminator() {
    let mut source = UtfString::<u16>::from_unicode_units(b"abc", -1);
    let pointer = source.to_c_ptr();
    let moved = UtfString::move_from(&mut source);
    assert_eq!(moved.to_c_ptr(), pointer);
    assert!(source.to_c_ptr().is_null());
    assert_eq!((source.size(), source.length()), (0, 0));
    assert!(source.iterator().buffer_here_ptr().is_null());
    source.clear();
    assert!(!source.to_c_ptr().is_null());
    assert_eq!(source.to_c_string(), [0]);
}

#[test]
fn move_assignment_swaps_buffers_and_returns_destination() {
    let mut first = UtfString::<u8>::from_unicode_units(b"first", -1);
    let mut second = UtfString::<u8>::from_unicode_units(b"two", -1);
    let pointers = (first.to_c_ptr(), second.to_c_ptr());
    let address = std::ptr::from_mut(&mut first);
    assert_eq!(std::ptr::from_mut(first.move_assign(&mut second)), address);
    assert_eq!(
        (first.to_c_ptr(), second.to_c_ptr()),
        (pointers.1, pointers.0)
    );
    assert_eq!(first.to_c_string(), b"two\0");
    assert_eq!(second.to_c_string(), b"first\0");
    assert_eq!(std::ptr::from_mut(first.move_assign_self()), address);
    assert_eq!(first.to_c_ptr(), pointers.1);
}

#[test]
fn copy_assignment_and_self_copy_keep_native_return_types_and_cached_lengths() {
    let source = UtfString::<u8>::from_unicode_units(&[0xD800_u16], -1);
    let mut destination = UtfString::<u8>::from_unicode_units(b"old", -1);
    let address = std::ptr::from_ref(&destination);
    assert_eq!(std::ptr::from_ref(destination.assign(&source)), address);
    assert_eq!((destination.size(), destination.length()), (0, 1));
    assert_ne!(destination.to_c_ptr(), source.to_c_ptr());
    let pointer = destination.to_c_ptr();
    assert_eq!(std::ptr::from_ref(destination.assign_self()), address);
    assert_eq!(destination.to_c_ptr(), pointer);
    destination.clone_from(&source);
    assert_eq!((destination.size(), destination.length()), (0, 1));
}

#[test]
fn returning_unicode_assignment_and_nullable_conversion_preserve_encoding() {
    let mut text = UtfString::<u16>::new();
    let address = std::ptr::from_ref(&text);
    assert_eq!(
        std::ptr::from_ref(text.assign_unicode("é".as_bytes())),
        address
    );
    assert_eq!(text.to_c_string(), [0xE9, 0]);
    text.assign_unicode(&[0x1F600 as WideChar]);
    assert_eq!(text.to_c_string(), [0xD83D, 0xDE00, 0]);
    text.from_optional_unicode::<u8>(None, -1);
    assert_eq!((text.size(), text.length()), (0, 0));
    assert!(!text.to_c_ptr().is_null());
    text.from_optional_unicode(Some(&[65_u32, 66]), 1);
    assert_eq!(text.to_c_string(), [65, 0]);
}

#[test]
fn self_buffer_copy_keeps_old_storage_alive_until_conversion_finishes() {
    let mut text = UtfString::<u8>::from_unicode_units("Aé😀z".as_bytes(), -1);
    text.from_self_unicode(1, 2);
    assert_eq!(text.to_c_string(), "é😀\0".as_bytes());
    assert_eq!((text.size(), text.length()), (6, 2));
    text.from_self_unicode(0, -1);
    assert_eq!(text.to_c_string(), "é😀\0".as_bytes());
    text.from_self_unicode(10, -1);
    assert_eq!(text.to_c_string(), b"\0");
}

#[test]
fn returning_append_and_self_append_preserve_empty_invalid_symbol_quirk() {
    let mut text = UtfString::<u8>::from_unicode_units(b"ab", -1);
    let suffix = UtfString::<u8>::from_unicode_units(b"c", -1);
    let address = std::ptr::from_mut(&mut text);
    assert_eq!(std::ptr::from_mut(text.append_returning(&suffix)), address);
    assert_eq!(std::ptr::from_mut(text.append_self()), address);
    assert_eq!(text.to_c_string(), b"abcabc\0");
    let mut invalid = UtfString::<u8>::from_unicode_units(&[0xD800_u16], -1);
    let pointer = invalid.to_c_ptr();
    invalid.append_self();
    assert_eq!(invalid.to_c_ptr(), pointer);
    assert_eq!((invalid.size(), invalid.length()), (0, 1));
}
