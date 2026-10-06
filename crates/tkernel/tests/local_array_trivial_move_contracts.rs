//! Trivial native heap-to-heap assignment swaps allocations and empties the source in place.
use rustycad_tkernel::ncollection::LocalArray;

fn initialized_local_move_array(size: usize, value: i32) -> LocalArray<i32, 4> {
    let mut array = LocalArray::new(size);
    array.as_mut_slice().fill(value);
    array
}

#[test]
fn trivial_heap_assignment_retains_the_old_destination_in_empty_source() {
    let mut destination = initialized_local_move_array(6, 11);
    let mut source = initialized_local_move_array(7, 22);
    let old_destination = destination.data_ptr();
    let old_source = source.data_ptr();
    let receiver = &destination as *const _;
    let returned = destination.move_from_trivial(&mut source);
    assert!(std::ptr::eq(receiver, returned));
    assert_eq!(returned.data_ptr(), old_source);
    assert_eq!(returned.as_slice(), &[22; 7]);
    assert_eq!(source.len(), 0);
    assert_eq!(source.data_ptr(), old_destination);
    assert_eq!(source.data_end_ptr(), old_destination);
}

#[test]
#[allow(unsafe_code)]
fn emptied_trivial_heap_source_keeps_its_initialized_allocation_alive() {
    let mut destination = initialized_local_move_array(6, 11);
    let mut source = initialized_local_move_array(7, 22);
    destination.move_from_trivial(&mut source);
    // SAFETY: the retained heap allocation still contains six initialized Copy elements.
    let retained = unsafe { source.data_ptr().read() };
    assert_eq!(retained, 11);
    source.allocate(2);
    assert_eq!(source.as_slice(), &[0, 0]);
    assert_eq!(destination.as_slice(), &[22; 7]);
}

#[test]
fn trivial_inline_source_copies_into_the_destination_inline_buffer() {
    for destination_size in [2, 6] {
        let mut destination = initialized_local_move_array(destination_size, 11);
        let mut source = initialized_local_move_array(3, 22);
        let source_pointer = source.data_ptr();
        destination.move_from_trivial(&mut source);
        assert_eq!(destination.as_slice(), &[22; 3]);
        assert_ne!(destination.data_ptr(), source_pointer);
        assert_eq!(source.data_ptr(), source_pointer);
        assert_eq!(source.len(), 0);
    }
}

#[test]
fn trivial_heap_source_transfers_into_an_inline_destination() {
    let mut destination = initialized_local_move_array(2, 11);
    let mut source = initialized_local_move_array(7, 22);
    let source_pointer = source.data_ptr();
    destination.move_from_trivial(&mut source);
    assert_eq!(destination.data_ptr(), source_pointer);
    assert_eq!(destination.as_slice(), &[22; 7]);
    assert_ne!(source.data_ptr(), source_pointer);
    assert_eq!(source.len(), 0);
}

#[test]
fn trivial_self_assignment_preserves_storage_and_returns_this_array() {
    let mut array = initialized_local_move_array(6, 11);
    let pointer = array.data_ptr();
    let receiver = &array as *const _;
    let returned = array.move_self_trivial();
    assert!(std::ptr::eq(receiver, returned));
    assert_eq!(returned.data_ptr(), pointer);
    assert_eq!(returned.as_slice(), &[11; 6]);
}

#[derive(Copy, Default)]
struct LocalMoveCopyOnly(i32);

// OCCT's trivial branch copies bytes and must not dispatch a user-defined copy hook.
#[allow(clippy::non_canonical_clone_impl)]
impl Clone for LocalMoveCopyOnly {
    fn clone(&self) -> Self {
        panic!("LocalMoveCopyOnly::clone must not run during the native memcpy branch");
    }
}

#[test]
fn trivial_inline_move_does_not_invoke_custom_clone_hooks() {
    let mut source = LocalArray::<LocalMoveCopyOnly, 4>::new(3);
    source.as_mut_slice().fill_with(|| LocalMoveCopyOnly(22));
    let mut destination = LocalArray::default();
    destination.move_from_trivial(&mut source);
    assert!(destination.iter().all(|element| element.0 == 22));
    assert_eq!(source.len(), 0);
}

#[test]
fn local_array_pointer_endpoints_cover_initialized_storage_and_empty_ranges() {
    let mut array = initialized_local_move_array(3, 11);
    let beginning = array.data_ptr();
    assert_eq!(array.data_end_ptr(), beginning.wrapping_add(3));
    assert_eq!(array.data_mut_ptr(), beginning.cast_mut());
    assert_eq!(
        array.data_end_mut_ptr(),
        beginning.wrapping_add(3).cast_mut()
    );
    array.deallocate();
    assert!(!array.data_ptr().is_null());
    assert_eq!(array.data_end_ptr(), array.data_ptr());
    assert_eq!(array.data_end_mut_ptr(), array.data_mut_ptr());
}

#[test]
fn owned_element_assignment_returns_destination_and_empties_source() {
    for (destination_size, source_size) in [(2, 3), (6, 3), (2, 7), (6, 7)] {
        let mut destination = LocalArray::<String, 4>::new(destination_size);
        let mut source = LocalArray::<String, 4>::new(source_size);
        source.as_mut_slice().fill_with(|| String::from("source"));
        let receiver = &destination as *const _;
        let returned = destination.move_from(&mut source);
        assert!(std::ptr::eq(receiver, returned));
        assert_eq!(returned.len(), source_size);
        assert!(returned.iter().all(|item| item == "source"));
        assert!(source.is_empty());
    }
}

#[test]
fn owned_element_self_assignment_preserves_the_receiver() {
    let mut array = LocalArray::<String, 4>::new(6);
    array.as_mut_slice().fill_with(|| String::from("preserved"));
    let pointer = array.data_ptr();
    let receiver = &array as *const _;
    let returned = array.move_self();
    assert!(std::ptr::eq(receiver, returned));
    assert_eq!(returned.data_ptr(), pointer);
    assert!(returned.iter().all(|item| item == "preserved"));
}
