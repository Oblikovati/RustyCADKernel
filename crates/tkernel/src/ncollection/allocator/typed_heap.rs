//! Layout-carrying Rust allocations do not need the raw OCCT allocation header.
use std::{
    alloc::{self, Layout},
    ptr::NonNull,
};

fn typed_layout(layout: Layout, minimum_alignment: usize) -> Option<Layout> {
    Layout::from_size_align(layout.size(), layout.align().max(minimum_alignment)).ok()
}

pub(super) fn allocate(layout: Layout, minimum_alignment: usize) -> Option<NonNull<u8>> {
    let layout = typed_layout(layout, minimum_alignment)?;
    // SAFETY: the Allocator implementation excludes zero sizes and validates alignment.
    NonNull::new(unsafe { alloc::alloc(layout) })
}

pub(super) unsafe fn deallocate(pointer: NonNull<u8>, layout: Layout, minimum_alignment: usize) {
    // Box can return a pointer reborrowed for T alone. Reading a hidden prefix through
    // that pointer violates provenance (caught by list_method_contracts under Miri).
    // The layout-bearing API can reconstruct the allocation without touching a header.
    let layout = typed_layout(layout, minimum_alignment).expect("layout validated at allocation");
    // SAFETY: caller returns this exact allocation with the layout used by allocate.
    unsafe { alloc::dealloc(pointer.as_ptr(), layout) };
}
