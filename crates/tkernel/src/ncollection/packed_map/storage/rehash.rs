//! Native words relink directly; open custom words retain transactional hash evaluation.
use super::super::{PackedWord, PackedWordReader};
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
        match T::Word::NATIVE_REHASH_READER {
            Some(reader) => self.rehash_native(&mut replacement, modulus, reader),
            None => self.rehash_custom(&mut replacement, modulus),
        }
        replacement
    }
    fn rehash_native(
        &mut self,
        replacement: &mut Self,
        modulus: NonZeroUsize,
        reader: PackedWordReader<T::Word>,
    ) {
        for bucket in 0..self.length {
            let mut pointer = self.head_ptr(bucket);
            self.set_head(bucket, std::ptr::null_mut());
            while !pointer.is_null() {
                // The private-constructor reader excludes user callbacks; target is in 1..=modulus.
                let mask = unsafe { (*pointer).mask() };
                let target = reader.bucket(mask, T::IS_64_BIT, modulus);
                pointer = replacement.prepend_existing(target, pointer);
            }
        }
    }
    fn rehash_custom(&mut self, replacement: &mut Self, modulus: NonZeroUsize) {
        // Public integer traits may panic. Resolve all hashes before changing any ownership.
        let destinations: Vec<usize> = self
            .nodes()
            .map(|node| node.hash_code(modulus.get()))
            .collect();
        let mut targets = destinations.into_iter();
        for bucket in 0..self.length {
            self.transfer_custom_chain(bucket, replacement, &mut targets);
        }
    }
    fn transfer_custom_chain(
        &mut self,
        bucket: usize,
        replacement: &mut Self,
        targets: &mut impl Iterator<Item = usize>,
    ) {
        let mut pointer = self.head_ptr(bucket);
        self.set_head(bucket, std::ptr::null_mut());
        while !pointer.is_null() {
            let target = targets.next().expect("one destination per owned node");
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
