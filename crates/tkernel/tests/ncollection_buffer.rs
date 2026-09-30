//! Empty buffer iterators must delimit an empty range even with an allocator.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{BaseAllocator, Buffer};
#[test]
fn NCollection_BufferTest__EmptyRange() {
    let mut buffer =
        Buffer::new(Some(BaseAllocator::common()), 0).expect("common Free accepts null");
    let bounds = buffer.as_mut_slice().as_mut_ptr_range();
    assert_eq!(bounds.start, bounds.end);
    let constant = buffer.as_slice().as_ptr_range();
    assert_eq!(constant.start, constant.end);
    assert_eq!(buffer.as_slice().iter().count(), 0);
}
