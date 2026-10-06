//! Rust Clone retains allocated-empty blocks and capacity; native Assign is a separate operation.
use super::TypedSparseStorage;
impl<T: Clone> Clone for TypedSparseStorage<T> {
    fn clone(&self) -> Self {
        let mut result = Self::new(self.base.block_size());
        result.assign(self);
        // assign reproduces allocated-empty source blocks; retained trailing table capacity is
        // also part of the prior Rust Clone extension, even when no occupied item reaches it.
        unsafe { result.base.reserve_clone_table(self.base.block_slots()) };
        result
    }
}
