//! Native copy-constructor failure has no container destructor to reclaim its raw storage.
use super::LinearVector;
use std::mem::ManuallyDrop;

impl<T: Clone> LinearVector<T> {
    pub(super) fn construct_copy(
        &self,
        #[cfg(test)] observe_failure: impl FnOnce(&mut Self),
    ) -> Self {
        // Native construction owns raw storage only after success. A failed constructor
        // intentionally leaves both this allocation and its completed prefix alive.
        let mut destination = ManuallyDrop::new(Self::with_capacity(self.len()));
        #[cfg(not(test))]
        destination.clone_unpublished_from(self, 0);
        #[cfg(test)]
        self.copy_observed_into(&mut destination, observe_failure);
        ManuallyDrop::into_inner(destination)
    }

    pub(super) fn clone_unpublished_from(&mut self, source: &Self, beginning: usize) {
        let mut index = beginning;
        self.construct_unpublished_tail(source.len(), || {
            let copied = source.elements[index].clone();
            index += 1;
            copied
        });
    }

    #[cfg(test)]
    fn copy_observed_into(&self, destination: &mut Self, observe_failure: impl FnOnce(&mut Self)) {
        use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
        let result = catch_unwind(AssertUnwindSafe(|| {
            destination.clone_unpublished_from(self, 0)
        }));
        if let Err(payload) = result {
            // Only tests inspect fresh allocation provenance after all element borrows end.
            // Observation performs no implicit cleanup; the same panic payload is resumed.
            observe_failure(destination);
            resume_unwind(payload);
        }
    }
}

#[cfg(test)]
mod tests;
