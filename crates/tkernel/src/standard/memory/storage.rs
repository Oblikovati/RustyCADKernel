//! Owned initialized bytes retain the original raw allocation pointer, including for zero bytes.
#![allow(unsafe_code)]
use super::raw_heap::{heap_allocate, heap_free, heap_reallocate};
use std::{fmt, ptr::NonNull};

pub(super) struct StandardByteStorage {
    pointer: NonNull<u8>,
    length: usize,
}
// SAFETY: every byte is initialized and exclusively owned; there is no allocator-thread affinity.
unsafe impl Send for StandardByteStorage {}
// SAFETY: shared access exposes only immutable bytes; resize and mutation require exclusive access.
unsafe impl Sync for StandardByteStorage {}

impl StandardByteStorage {
    pub(super) fn allocate(size: usize, alignment: usize, zeroed: bool) -> Option<Self> {
        let pointer = heap_allocate(size, alignment, zeroed)?;
        if !zeroed {
            // SAFETY: fresh malloc-like storage must be initialized before safe byte slices exist.
            unsafe { pointer.write_bytes(0, size) };
        }
        Some(Self {
            pointer,
            length: size,
        })
    }

    pub(super) fn as_slice(&self) -> &[u8] {
        // SAFETY: the original allocation remains live and all requested bytes are initialized.
        unsafe { std::slice::from_raw_parts(self.pointer.as_ptr(), self.length) }
    }

    pub(super) fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: the exclusive owner borrow excludes overlapping payload reads and writes.
        unsafe { std::slice::from_raw_parts_mut(self.pointer.as_ptr(), self.length) }
    }

    pub(super) fn len(&self) -> usize {
        // Logical length remains the requested extent, independent of private heap headers.
        self.length
    }

    pub(super) fn is_empty(&self) -> bool {
        // Zero bytes still own a distinct actual raw allocation that Drop must release.
        self.length == 0
    }

    pub(super) fn resize(&mut self, size: usize) -> Option<()> {
        // SAFETY: retain the original allocation provenance, never a retagged slice pointer.
        let pointer = unsafe { heap_reallocate(self.pointer, size) }?;
        if size > self.length {
            // SAFETY: only the newly grown suffix is uninitialized; the old prefix remains intact.
            unsafe { pointer.add(self.length).write_bytes(0, size - self.length) };
        }
        self.pointer = pointer;
        self.length = size;
        Some(())
    }
}

impl Drop for StandardByteStorage {
    fn drop(&mut self) {
        // SAFETY: this is the retained original live allocation; no other owner can free it.
        unsafe { heap_free(self.pointer) };
    }
}
impl PartialEq for StandardByteStorage {
    fn eq(&self, other: &Self) -> bool {
        // StandardMemory previously compared Vec contents; physical allocation identity is separate.
        self.as_slice() == other.as_slice()
    }
}
impl Eq for StandardByteStorage {}
impl fmt::Debug for StandardByteStorage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Retain StandardMemory's byte-list Debug representation without exposing heap metadata.
        self.as_slice().fmt(formatter)
    }
}

#[cfg(test)]
mod tests;
