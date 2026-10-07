//! Native ReSize relinks each chain in old-bucket order into the new table.
use super::{PackedInteger, PackedMapNode, PackedNodeTable};
use std::num::NonZeroUsize;
fn checked_modulus(modulus: usize) -> NonZeroUsize {
    NonZeroUsize::new(modulus)
        .unwrap_or_else(|| panic!("bucket count {modulus}; expected a value in 1..=usize::MAX"))
}
impl<T: PackedInteger> PackedNodeTable<T> {
    pub(in super::super) fn rehashed(&mut self, modulus: usize) -> Self {
        let modulus = checked_modulus(modulus);
        let length = modulus.get().checked_add(1).unwrap_or_else(|| {
            panic!("bucket count {modulus}; expected space for the terminal slot")
        });
        let mut replacement = Self::allocated(length);
        for bucket in 0..self.length {
            self.transfer_chain(bucket, &mut replacement, modulus);
        }
        replacement
    }
    fn transfer_chain(&mut self, bucket: usize, replacement: &mut Self, modulus: NonZeroUsize) {
        let mut pointer = self.head_ptr(bucket);
        self.set_head(bucket, std::ptr::null_mut());
        while !pointer.is_null() {
            // Sealed native words: the hash is a plain cast and cannot unwind mid-relink.
            let target = unsafe { (*pointer).hash_code(modulus.get()) };
            pointer = replacement.prepend_existing(target, pointer);
        }
    }
    fn prepend_existing(
        &mut self,
        target: usize,
        pointer: *mut PackedMapNode<T>,
    ) -> *mut PackedMapNode<T> {
        // No user callback occurs between unlinking and linking this exclusively owned node.
        let node = unsafe { &mut *pointer };
        let next = node.next();
        node.set_next(self.head_ptr(target));
        self.set_head(target, pointer);
        next
    }
}
