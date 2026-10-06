//! Shared native unlink/assignment/relink ordering, with explicit Rust unwind cleanup.
use super::{IndexedNode, IndexedStorage};
use crate::{
    ncollection::{HashPolicy, destructor_unwind::DeferredDestructorPanic},
    standard::Failure,
};
impl<N: IndexedNode> IndexedStorage<N>
where
    N::Key: std::fmt::Debug,
{
    pub(in crate::ncollection) fn prepare_substitute(
        &mut self,
        index: usize,
        key: &N::Key,
        policy: &impl HashPolicy<N::Key>,
        kind: &str,
    ) -> Result<(usize, bool), Failure> {
        self.check_index(index, kind)?;
        let (existing, bucket) = self.lookup(key, policy);
        Self::check_substitute(index, existing, key, kind)?;
        if existing == 0 {
            self.unlink(
                policy.hash(self.key(index)) % self.bucket_count() + 1,
                self.node(index),
            );
        }
        Ok((bucket, existing == 0))
    }
    fn check_substitute(
        index: usize,
        existing: usize,
        key: &N::Key,
        kind: &str,
    ) -> Result<(), Failure> {
        if existing == 0 || existing == index {
            return Ok(());
        }
        Err(Failure::with_type(
            "Standard_DomainError",
            &format!(
                "{kind} key {key:?} already at {existing}; expected an absent key or index {index}"
            ),
            "",
        ))
    }
}
impl<N: IndexedNode> IndexedStorage<N> {
    pub(in crate::ncollection) fn replace_substitute(
        &mut self,
        index: usize,
        bucket: usize,
        detached: bool,
        assign: impl FnOnce(*mut N),
    ) {
        let node = self.node(index);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| assign(node.cast())));
        if detached {
            self.prepend(bucket, node);
        }
        if let Err(panic) = result {
            self.clear_failed_assignment(panic);
        }
    }
    fn clear_failed_assignment(&mut self, panic: Box<dyn std::any::Any + Send>) {
        // A throwing clone_from can change the hash key partially. Clearing is a Rust safety
        // extension; native nonthrowing assignment/unlink/relink order remains unchanged.
        let mut panics = DeferredDestructorPanic::default();
        panics.capture(|| std::panic::resume_unwind(panic));
        panics.capture(|| self.clear(false));
        panics.finish(Ok(()));
    }
}
