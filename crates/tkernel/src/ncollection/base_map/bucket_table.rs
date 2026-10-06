//! Bucket allocations use Standard storage independently of the node allocator.
#![allow(unsafe_code)]
use super::MapLinkNode;
use crate::{ncollection::StandardAllocator, standard::Failure};
use std::ptr::NonNull;

pub(super) struct BucketTable {
    pointer: NonNull<*mut MapLinkNode>,
    count: usize,
}

impl BucketTable {
    pub(super) fn zeroed(count: usize) -> Result<Self, Failure> {
        validate_count(count)?;
        let pointer = StandardAllocator::new()
            .allocate(count)
            .ok_or_else(|| allocation_error(count))?;
        let mut table = Self { pointer, count };
        table.initialize_nulls(0);
        Ok(table)
    }
    pub(super) fn as_ptr(&self) -> *mut *mut MapLinkNode {
        self.pointer.as_ptr()
    }
    pub(super) fn as_slice(&self) -> &[*mut MapLinkNode] {
        // SAFETY: every slot is initialized and self owns count live pointer slots.
        unsafe { std::slice::from_raw_parts(self.pointer.as_ptr(), self.count) }
    }
    pub(super) fn as_mut_slice(&mut self) -> &mut [*mut MapLinkNode] {
        // SAFETY: the exclusive owner borrow prevents competing slice access.
        unsafe { std::slice::from_raw_parts_mut(self.pointer.as_ptr(), self.count) }
    }
    pub(super) fn initialize_nulls(&mut self, start: usize) {
        // SAFETY: callers provide start <= count; null is a valid initialized pointer value.
        unsafe {
            self.pointer
                .as_ptr()
                .add(start)
                .write_bytes(0, self.count - start)
        };
    }
    pub(super) fn resize(&mut self, count: usize) -> Result<(), Failure> {
        validate_count(count)?;
        // SAFETY: pointer is this StandardAllocator's live allocation; count is nonzero.
        let resized = unsafe { StandardAllocator::new().reallocate(self.pointer, count) };
        self.pointer = resized.ok_or_else(|| allocation_error(count))?;
        let old_count = std::mem::replace(&mut self.count, count);
        self.initialize_nulls(old_count.min(count));
        Ok(())
    }
}

impl Drop for BucketTable {
    fn drop(&mut self) {
        // SAFETY: sole owner releases precisely the raw Standard allocation, never the nodes.
        unsafe { StandardAllocator::new().deallocate(self.pointer, self.count) }
            .expect("BaseMap bucket storage: live Standard allocation expected");
    }
}

fn validate_count(count: usize) -> Result<(), Failure> {
    if count == 0 || count > isize::MAX as usize / size_of::<*mut MapLinkNode>() {
        return Err(allocation_error(count));
    }
    Ok(())
}

fn allocation_error(count: usize) -> Failure {
    Failure::with_message(
        "Standard_OutOfMemory",
        &format!(
            "BaseMap bucket allocation of {count} pointers failed; expected nonzero addressable table"
        ),
    )
}
