#![allow(unsafe_code)]
//! Authoritative native bucket heads and globally allocated, stable-address node chains.
use super::{PackedInteger, PackedMapNode};
use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::ptr::NonNull;
mod mutation;

pub(super) struct PackedNodeTable<T: PackedInteger> {
    pub(super) heads: NonNull<*mut PackedMapNode<T>>,
    length: usize,
}
impl<T: PackedInteger> Default for PackedNodeTable<T> {
    fn default() -> Self {
        Self {
            heads: NonNull::dangling(),
            length: 0,
        }
    }
}
// The table exclusively owns every reachable node. Links cannot escape as mutable references.
// Only the word is stored inside nodes; PackedMap's PhantomData separately tracks its key type.
unsafe impl<T: PackedInteger> Send for PackedNodeTable<T> where T::Word: Send {}
unsafe impl<T: PackedInteger> Sync for PackedNodeTable<T> where T::Word: Sync {}
impl<T: PackedInteger> PackedNodeTable<T> {
    fn allocated(length: usize) -> Self {
        let layout = Self::layout(length);
        // Pinned Standard::AllocateOptimal/Free use the native heap. Keep the table's
        // allocation separate from global node Box allocation and retain pointer alignment.
        let raw = unsafe { alloc(layout) };
        let heads = NonNull::new(raw.cast()).unwrap_or_else(|| handle_alloc_error(layout));
        let mut table = Self { heads, length };
        for bucket in 0..length {
            table.set_head(bucket, std::ptr::null_mut());
        }
        table
    }
    fn layout(length: usize) -> Layout {
        Layout::array::<*mut PackedMapNode<T>>(length).unwrap_or_else(|_| {
            panic!("bucket count {length}: expected a representable pointer array")
        })
    }
    pub(super) fn len(&self) -> usize {
        self.length
    }
    fn head_ptr(&self, bucket: usize) -> *mut PackedMapNode<T> {
        assert!(
            bucket < self.length,
            "bucket {bucket}: expected index below {}",
            self.length
        );
        // Every allocated slot is initialized, and the table retains its allocation.
        unsafe { self.heads.as_ptr().add(bucket).read() }
    }
    fn set_head(&mut self, bucket: usize, node: *mut PackedMapNode<T>) {
        assert!(
            bucket < self.length,
            "bucket {bucket}: expected index below {}",
            self.length
        );
        // Exclusive table access protects the initialized pointer slot.
        unsafe { self.heads.as_ptr().add(bucket).write(node) };
    }
    pub(super) fn head(&self, bucket: usize) -> Option<&PackedMapNode<T>> {
        if bucket >= self.length {
            return None;
        }
        // Links are installed only from live global Box allocations owned by this table.
        unsafe { self.head_ptr(bucket).as_ref() }
    }
    pub(super) fn following<'a>(
        &'a self,
        node: &'a PackedMapNode<T>,
    ) -> Option<&'a PackedMapNode<T>> {
        // Only internal callers supply nodes borrowed from this same immutable table.
        unsafe { node.next().as_ref() }
    }
    pub(super) fn chain(&self, bucket: usize) -> impl Iterator<Item = &PackedMapNode<T>> {
        std::iter::successors(self.head(bucket), |node| self.following(node))
    }
    pub(super) fn nodes(&self) -> impl Iterator<Item = &PackedMapNode<T>> {
        (0..self.length).flat_map(|bucket| self.chain(bucket))
    }
    pub(super) fn find(&self, bucket: usize, index: T::Word) -> Option<&PackedMapNode<T>> {
        self.chain(bucket).find(|node| node.is_equal(index))
    }
    pub(super) fn find_mut(
        &mut self,
        bucket: usize,
        index: T::Word,
    ) -> Option<&mut PackedMapNode<T>> {
        let mut pointer = self.head_ptr(bucket);
        // Exclusive table access makes each reachable node exclusively accessible.
        while let Some(node) = unsafe { pointer.as_mut() } {
            if node.is_equal(index) {
                return Some(node);
            }
            pointer = node.next();
        }
        None
    }
}
impl<T: PackedInteger> Drop for PackedNodeTable<T> {
    fn drop(&mut self) {
        for bucket in 0..self.length {
            let mut pointer = self.head_ptr(bucket);
            while !pointer.is_null() {
                // Remove exactly one owned global allocation; Next itself is non-owning.
                let owner = unsafe { Box::from_raw(pointer) };
                pointer = owner.next();
            }
        }
        if self.length != 0 {
            // This is the identical layout used for the separate optimal bucket allocation.
            unsafe { dealloc(self.heads.as_ptr().cast(), Self::layout(self.length)) };
        }
    }
}
