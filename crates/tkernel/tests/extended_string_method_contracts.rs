//! Native ExtendedString shared-empty and ownership method contracts.
use rustycad_tkernel::tcollection::ExtendedString;

#[test]
fn independently_constructed_empty_strings_share_the_native_terminator() {
    let first = ExtendedString::default();
    let second = ExtendedString::from_raw(&[]);
    assert_eq!(first.to_ext_ptr(), second.to_ext_ptr());
    assert_eq!(
        first.to_ext_ptr(),
        ExtendedString::empty_string().to_ext_ptr()
    );
    assert_eq!(first.as_utf16_with_nul(), [0]);
    assert_eq!(
        ExtendedString::filled(0, 42).unwrap().to_ext_ptr(),
        first.to_ext_ptr()
    );
    assert_eq!(first.clone().to_ext_ptr(), first.to_ext_ptr());
}

#[test]
fn shortening_keeps_owned_empty_storage_while_clear_releases_it() {
    let mut text = ExtendedString::new("abc");
    let owned = text.to_ext_ptr();
    text.trunc(0).unwrap();
    assert_eq!(text.to_ext_ptr(), owned);
    assert_ne!(owned, ExtendedString::default().to_ext_ptr());
    assert_eq!(text, ExtendedString::default());
    assert_eq!(
        text.clone().to_ext_ptr(),
        ExtendedString::default().to_ext_ptr()
    );
    text.clear();
    assert_eq!(text.to_ext_ptr(), ExtendedString::default().to_ext_ptr());
}

#[test]
fn empty_copy_paths_retain_destination_buffer_even_for_null_and_negative_spans() {
    let mut text = ExtendedString::new("abc");
    let pointer = text.to_ext_ptr();
    text.copy_span(None, 20);
    assert_eq!(text.to_ext_ptr(), pointer);
    text.copy_span(Some(&[65]), -1);
    assert_eq!(text.to_ext_ptr(), pointer);
    text.copy_assign(&ExtendedString::default());
    assert_eq!(text.to_ext_ptr(), pointer);
    text.copy_c_units(None);
    assert_eq!(text.to_ext_ptr(), pointer);
    assert_eq!(text.as_utf16_with_nul(), [0]);
}

#[test]
fn move_constructor_and_assignment_transfer_allocated_empty_storage() {
    let mut source = ExtendedString::new("abc");
    let pointer = source.to_ext_ptr();
    source.trunc(0).unwrap();
    let mut moved = ExtendedString::move_from(&mut source);
    assert_eq!(moved.to_ext_ptr(), pointer);
    assert_eq!(source.to_ext_ptr(), ExtendedString::default().to_ext_ptr());
    let mut destination = ExtendedString::new("old");
    let address = std::ptr::from_mut(&mut destination);
    assert_eq!(
        std::ptr::from_mut(destination.move_assign_returning(&mut moved)),
        address
    );
    assert_eq!(destination.to_ext_ptr(), pointer);
    assert_eq!(moved.to_ext_ptr(), source.to_ext_ptr());
    assert_eq!(std::ptr::from_mut(destination.move_assign_self()), address);
    assert_eq!(destination.to_ext_ptr(), pointer);
}

#[test]
fn moving_static_empty_source_keeps_destination_allocation() {
    let mut source = ExtendedString::default();
    let shared = source.to_ext_ptr();
    let mut destination = ExtendedString::new("old");
    let pointer = destination.to_ext_ptr();
    destination.move_assign(&mut source);
    assert_eq!(destination.to_ext_ptr(), pointer);
    assert!(destination.is_empty());
    assert_eq!(source.to_ext_ptr(), shared);
    assert_eq!(ExtendedString::move_from(&mut source).to_ext_ptr(), shared);
}

#[test]
fn remove_all_compacts_in_place_without_replacing_the_native_buffer() {
    let mut text = ExtendedString::from_raw(&[65, 0, 66, 65]);
    let pointer = text.to_ext_ptr();
    text.remove_all(65);
    assert_eq!(text.as_utf16(), [0, 66]);
    assert_eq!(text.to_ext_ptr(), pointer);
    text.remove_all(0);
    text.remove_all(66);
    assert!(text.is_empty());
    assert_eq!(text.to_ext_ptr(), pointer);
}

#[test]
fn returning_view_assignment_preserves_embedded_zero_units() {
    let mut text = ExtendedString::default();
    let address = std::ptr::from_mut(&mut text);
    assert_eq!(std::ptr::from_mut(text.assign_view(&[65, 0, 66])), address);
    assert_eq!(text.as_utf16(), [65, 0, 66]);
    let pointer = text.to_ext_ptr();
    text.assign_view(&[]);
    assert_eq!(text.to_ext_ptr(), pointer);
}

#[test]
fn no_op_mutations_preserve_static_empty_storage_and_real_mutations_initialize_it() {
    let mut text = ExtendedString::default();
    let shared = text.to_ext_ptr();
    text.right_adjust();
    text.left_adjust();
    text.remove_all(42);
    text.trunc(0).unwrap();
    text.set_value_units(1, []).unwrap();
    text.insert_span(1, None, 1).unwrap();
    text.center(0, 42).unwrap();
    text.left_justify(0, 42).unwrap();
    text.right_justify(0, 42).unwrap();
    assert_eq!(text.to_ext_ptr(), shared);
    text.insert_char(1, 42).unwrap();
    assert_eq!(text.as_utf16_with_nul(), [42, 0]);
}

#[test]
fn empty_string_growth_initializes_terminator_for_each_edit_family() {
    let mut append = ExtendedString::default();
    append.assign_cat(65_u16);
    let mut prepend = ExtendedString::default();
    prepend.prepend([65]);
    let mut insert = ExtendedString::default();
    insert.insert(1, 65_u16).unwrap();
    let mut left = ExtendedString::default();
    left.left_justify(1, 65).unwrap();
    let mut right = ExtendedString::default();
    right.right_justify(1, 65).unwrap();
    let mut center = ExtendedString::default();
    center.center(1, 65).unwrap();
    for text in [append, prepend, insert, left, right, center] {
        assert_eq!(text.as_utf16_with_nul(), [65, 0]);
    }
}
