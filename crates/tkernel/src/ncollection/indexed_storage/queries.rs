//! Both indexed families use the same native key-chain lookup and validated dense slots.
use super::{IndexedNode, IndexedStorage};
use crate::{ncollection::HashPolicy, standard::Failure};
impl<N: IndexedNode> IndexedStorage<N> {
    pub(in crate::ncollection) fn lookup(
        &self,
        key: &N::Key,
        policy: &impl HashPolicy<N::Key>,
    ) -> (usize, usize) {
        let bucket = policy.hash(key) % self.bucket_count() + 1;
        let mut node = self.head(bucket);
        while !node.is_null() {
            let stored = unsafe { &*N::key_ptr(node.cast()) };
            if policy.equal(stored, key) {
                return (unsafe { *N::index_ptr(node.cast()) } as usize, bucket);
            }
            node = unsafe { (*node).next() };
        }
        (0, bucket)
    }
    pub(in crate::ncollection) fn check_index(
        &self,
        index: usize,
        kind: &str,
    ) -> Result<(), Failure> {
        if index == 0 || index > self.len() {
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!("{kind} index {index}; expected 1..={}", self.len()),
                "",
            ));
        }
        Ok(())
    }
    pub(in crate::ncollection) fn value(&self, index: usize) -> &N::Value {
        unsafe { &*N::value_ptr(self.node(index).cast()) }
    }
    pub(in crate::ncollection) fn value_mut(&mut self, index: usize) -> &mut N::Value {
        unsafe { &mut *N::value_ptr(self.node(index).cast()) }
    }
    pub(in crate::ncollection) fn remove_last(&mut self, policy: &impl HashPolicy<N::Key>) {
        let node = self.node(self.len());
        self.set_last_slot(std::ptr::null_mut());
        let hashed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            policy.hash(unsafe { &*N::key_ptr(node.cast()) }) % self.bucket_count() + 1
        }));
        let bucket = match hashed {
            Ok(bucket) => bucket,
            Err(panic) => {
                self.set_last_slot(node);
                std::panic::resume_unwind(panic)
            }
        };
        self.destroy_last(bucket, node);
    }
}
