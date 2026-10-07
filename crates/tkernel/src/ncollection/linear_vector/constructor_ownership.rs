//! Constructors transfer raw storage ownership only when initialization completes.
use super::LinearVector;
use std::mem::ManuallyDrop;

impl<T> LinearVector<T> {
    pub(super) fn construct_owned(
        capacity: usize,
        initialize: impl FnOnce(&mut Self),
        #[cfg(test)] observe_failure: impl FnOnce(&mut Self),
    ) -> Self {
        // Native construction owns raw storage only after success. A failed constructor
        // intentionally leaves both this allocation and its completed prefix alive.
        let mut destination = ManuallyDrop::new(Self::with_capacity(capacity));
        #[cfg(not(test))]
        initialize(&mut destination);
        #[cfg(test)]
        destination.initialize_observed(initialize, observe_failure);
        ManuallyDrop::into_inner(destination)
    }

    #[cfg(test)]
    fn initialize_observed(
        &mut self,
        initialize: impl FnOnce(&mut Self),
        observe_failure: impl FnOnce(&mut Self),
    ) {
        use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
        let result = catch_unwind(AssertUnwindSafe(|| initialize(self)));
        if let Err(payload) = result {
            // Only tests inspect fresh allocation provenance after all element borrows end.
            // Observation performs no implicit cleanup; the same panic payload is resumed.
            observe_failure(self);
            resume_unwind(payload);
        }
    }
}
