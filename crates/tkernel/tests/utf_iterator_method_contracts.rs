//! Pointer identity, null state and output position contracts from NCollection_UtfIterator.
use rustycad_tkernel::ncollection::UtfIterator;

#[test]
fn null_constructor_and_init_preserve_both_null_positions() {
    let mut iterator = UtfIterator::<u8>::from_optional(None);
    assert!(iterator.buffer_here_ptr().is_null());
    assert!(iterator.buffer_next_ptr().is_null());
    assert_eq!((*iterator, iterator.index()), (0, 0));
    assert!(iterator.is_valid());
    iterator.init(b"ab\0");
    iterator.advance();
    iterator.init_optional(None);
    assert!(iterator.buffer_here_ptr().is_null());
    assert!(iterator.buffer_next_ptr().is_null());
    assert_eq!((*iterator, iterator.index()), (0, 0));
    assert_eq!(iterator, UtfIterator::from_optional(None));
}

#[test]
fn equal_addresses_compare_equal_across_different_slice_origins() {
    let bytes = b"a\xc3\xa9z\0";
    let mut complete = UtfIterator::new(bytes);
    complete.advance();
    let suffix = UtfIterator::new(&bytes[1..]);
    assert_eq!(complete, suffix);
    assert_eq!((complete.index(), suffix.index()), (1, 0));
    assert_eq!(complete.buffer_here_ptr(), bytes.as_ptr().wrapping_add(1));
    assert_eq!(complete.buffer_next_ptr(), bytes.as_ptr().wrapping_add(3));
    assert_eq!(*complete, 0xe9);
}

#[test]
fn wide_pointers_advance_in_code_units_and_keep_postfix_state() {
    let units = [0xd83d_u16, 0xde00, 0x41, 0];
    let mut iterator = UtfIterator::new(&units);
    assert_eq!(iterator.buffer_next_ptr(), units.as_ptr().wrapping_add(2));
    let before = iterator.post_increment();
    assert_eq!(*before, 0x1f600);
    assert_eq!(iterator.buffer_here_ptr(), units.as_ptr().wrapping_add(2));
    assert_eq!(iterator, UtfIterator::new(&units[2..]));
    let scalar = [0x1f600_u32, 0];
    let scalar_iterator = UtfIterator::new(&scalar);
    assert_eq!(
        scalar_iterator.buffer_next_ptr(),
        scalar.as_ptr().wrapping_add(1)
    );
}

#[test]
fn output_suffix_matches_native_next_pointer_and_rejects_short_buffer() {
    let units = [0x1f600_u32];
    let iterator = UtfIterator::new(&units);
    let mut output = [7_u8; 6];
    let origin = output.as_mut_ptr();
    let remaining = iterator.get_utf_remaining(&mut output).unwrap();
    assert_eq!(remaining.as_mut_ptr(), origin.wrapping_add(4));
    assert_eq!(remaining, [7, 7]);
    assert_eq!(output[..4], [0xf0, 0x9f, 0x98, 0x80]);
    assert_eq!(
        iterator.get_utf_remaining(&mut [0_u16]).unwrap_err().needed,
        2
    );
}

#[test]
fn invalid_symbol_preserves_output_position_and_null_differs_from_empty() {
    let invalid = [0x110000_u32];
    let iterator = UtfIterator::new(&invalid);
    let mut output = [9_u16; 2];
    let origin = output.as_mut_ptr();
    assert_eq!(
        iterator
            .get_utf_remaining(&mut output)
            .unwrap()
            .as_mut_ptr(),
        origin
    );
    assert_eq!(output, [9, 9]);
    let null = UtfIterator::<u8>::from_optional(None);
    let empty = UtfIterator::new(&[]);
    assert_ne!(null, empty);
    assert!(!empty.buffer_here_ptr().is_null());
}

#[test]
#[should_panic(expected = "UtfIterator buffer is null; expected a non-null code-unit buffer")]
fn null_increment_is_rejected_instead_of_dereferencing_null() {
    UtfIterator::<u8>::from_optional(None).advance();
}
