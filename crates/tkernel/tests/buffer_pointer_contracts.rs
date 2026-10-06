//! Native NCollection_Buffer iterator endpoints preserve null and allocated-zero identities.
use rustycad_tkernel::ncollection::{BaseAllocator, Buffer};

#[test]
fn unallocated_buffer_endpoints_are_null() {
    let mut buffer = Buffer::new(None, 0).unwrap();
    assert!(buffer.data_ptr().is_null());
    assert!(buffer.data_mut_ptr().is_null());
    assert!(buffer.data_end_ptr().is_null());
    assert!(buffer.data_end_mut_ptr().is_null());
}

#[test]
fn allocated_zero_endpoints_equal_the_nonnull_allocation() {
    let mut buffer = Buffer::new(Some(BaseAllocator::common()), 0).unwrap();
    let beginning = buffer.data_ptr();
    assert!(!beginning.is_null());
    assert_eq!(buffer.data_end_ptr(), beginning);
    assert_eq!(buffer.data_mut_ptr(), beginning.cast_mut());
    assert_eq!(buffer.data_end_mut_ptr(), beginning.cast_mut());
}

#[test]
fn owned_endpoints_delimit_the_full_byte_span() {
    let mut buffer = Buffer::new(Some(BaseAllocator::common()), 4).unwrap();
    let beginning = buffer.data_ptr();
    let ending = beginning.wrapping_add(4);
    assert_eq!(buffer.data_end_ptr(), ending);
    assert_eq!(buffer.data_mut_ptr(), beginning.cast_mut());
    assert_eq!(buffer.data_end_mut_ptr(), ending.cast_mut());
    buffer.as_mut_slice().copy_from_slice(b"abcd");
    assert_eq!(buffer.data_ptr(), beginning);
    assert_eq!(buffer.as_slice(), b"abcd");
}

#[test]
#[allow(unsafe_code)]
fn mutable_buffer_pointer_writes_the_owned_byte_span() {
    let mut buffer = Buffer::new(Some(BaseAllocator::common()), 4).unwrap();
    let beginning = buffer.data_mut_ptr();
    // SAFETY: four initialized bytes belong exclusively to this live buffer; no slices exist.
    unsafe { beginning.write_bytes(b'x', 4) };
    assert_eq!(buffer.as_slice(), b"xxxx");
    assert_eq!(buffer.data_end_mut_ptr(), beginning.wrapping_add(4));
}

#[test]
fn freeing_and_reallocating_reset_pointer_endpoints() {
    let mut buffer = Buffer::new(Some(BaseAllocator::common()), 4).unwrap();
    buffer.free().unwrap();
    assert!(buffer.data_end_ptr().is_null());
    assert!(buffer.data_end_mut_ptr().is_null());
    assert!(buffer.data_mut_ptr().is_null());
    assert!(buffer.allocate(3).unwrap());
    assert_eq!(buffer.data_end_ptr(), buffer.data_ptr().wrapping_add(3));
}

#[test]
fn adopted_storage_keeps_its_original_pointer_range() {
    let allocator = BaseAllocator::common();
    let allocation = allocator.allocate(3).unwrap();
    let beginning = allocation.as_ptr();
    let mut buffer = Buffer::with_data(Some(allocator), allocation);
    assert_eq!(buffer.data_ptr(), beginning);
    assert_eq!(buffer.data_end_ptr(), beginning.wrapping_add(3));
    assert_eq!(buffer.data_mut_ptr(), beginning.cast_mut());
    assert_eq!(
        buffer.data_end_mut_ptr(),
        beginning.wrapping_add(3).cast_mut()
    );
}
