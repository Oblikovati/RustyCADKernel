//! An owning boundary prevents safe access to native destructor-interrupted storage.
#![allow(unsafe_code)]
use super::{LinearVector, relocation::next_capacity_for};
use std::{mem::ManuallyDrop, ops::Range};
mod raw_storage;
use raw_storage::InterruptedGrowth;

/// Logical native storage state; `token.progress().dead_slots.is_empty()` checks liveness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrowthProgress {
    /// Native registered size, including slots whose lifetimes ended on a destructor panic.
    pub size: usize,
    /// Capacity of the currently published source storage.
    pub capacity: usize,
    /// Slots requiring explicit reconstruction before the original vector can be recovered.
    pub dead_slots: Range<usize>,
}

enum GrowthStorage<T> {
    Live(LinearVector<T>),
    Interrupted(InterruptedGrowth<T>),
}

/// Owns native growth across destructor panic without exposing dead slots through safe APIs.
/// `let token = LinearVector::<i32>::default().into_growth_token();`
/// An interrupted token deliberately leaks its allocations if dropped without repair.
/// This safety fallback receives no native destructor-behavior equivalence claim.
/// Reconstruct holes explicitly before extracting the original vector; repair leaves
/// the native abandoned replacement allocation alone.
pub struct LinearVectorGrowth<T> {
    current: Option<GrowthStorage<T>>,
    abandoned: Option<ManuallyDrop<LinearVector<T>>>,
}

impl<T> LinearVector<T> {
    /// Transfers ownership to a native growth token; `let token = vector.into_growth_token();`.
    /// Existing safe reserve operations retain their ordinary Rust destructor cleanup policy.
    pub fn into_growth_token(self) -> LinearVectorGrowth<T> {
        LinearVectorGrowth {
            current: Some(GrowthStorage::Live(self)),
            abandoned: None,
        }
    }
}

impl<T> LinearVectorGrowth<T> {
    /// Reads metadata without reading elements; `assert_eq!(token.progress().size, 3);`.
    pub fn progress(&self) -> GrowthProgress {
        match self
            .current
            .as_ref()
            .expect("growth ownership: expected current storage")
        {
            GrowthStorage::Live(vector) => GrowthProgress {
                size: vector.len(),
                capacity: vector.capacity(),
                dead_slots: 0..0,
            },
            GrowthStorage::Interrupted(storage) => storage.progress(),
        }
    }

    /// Returns a raw address only; `let address = token.original_ptr();`.
    /// Dead slots identified by `progress()` must never be read or dropped as T values.
    pub fn original_ptr(&self) -> *const T {
        match self
            .current
            .as_ref()
            .expect("growth ownership: expected current storage")
        {
            GrowthStorage::Live(vector) => vector.data_ptr(),
            GrowthStorage::Interrupted(storage) => storage.original_ptr(),
        }
    }

    /// Returns the vector only after every original slot is live; `token.try_into_vector()`.
    /// Interrupted tokens return themselves, preserving the sole repair obligation.
    pub fn try_into_vector(mut self) -> Result<LinearVector<T>, Self> {
        if matches!(self.current, Some(GrowthStorage::Interrupted(_))) {
            return Err(self);
        }
        Ok(self.take_live())
    }

    /// Grows with native original-slot destruction; `unsafe { token.reserve(8, |value| *value) }`.
    /// Relocation must leave its source valid on unwind. A destructor panic preserves
    /// native metadata and marks ended lifetimes; repair is required before extraction
    /// or another reserve. Address-dependent constructors and receiver aliases are excluded.
    /// # Safety
    /// No live references may alias original slots or owned payload storage whose
    /// lifetimes destruction ends. Callback observers must not alias the exclusively
    /// borrowed token or receiver. Every relocation must leave its source initialized
    /// even on unwind. A throwing destructor ends its slot lifetime, including fields;
    /// the token tracks that slot as dead and prevents extraction until explicit repair.
    pub unsafe fn reserve(&mut self, minimum: usize, relocate: impl FnMut(&mut T) -> T) {
        self.reserve_owned(
            minimum,
            relocate,
            #[cfg(test)]
            |_| {},
        );
    }

    /// Explicitly reconstructs dead slots; `token.repair_with(|index| index as i32)`.
    /// A constructor panic preserves completed repairs and the remaining hole range.
    /// This never destroys or frees the native abandoned replacement allocation.
    pub fn repair_with(&mut self, construct: impl FnMut(usize) -> T) {
        let Some(GrowthStorage::Interrupted(storage)) = self.current.as_mut() else {
            return;
        };
        storage.repair_with(construct);
        let storage = self.take_interrupted();
        self.current = Some(GrowthStorage::Live(LinearVector {
            elements: storage.into_initialized(),
        }));
    }

    fn reserve_owned(
        &mut self,
        minimum: usize,
        mut relocate: impl FnMut(&mut T) -> T,
        #[cfg(test)] observe_failure: impl FnOnce(&mut LinearVector<T>),
    ) {
        if minimum <= self.live_mut().capacity() {
            return;
        }
        let replacement = self.construct_replacement(
            minimum,
            &mut relocate,
            #[cfg(test)]
            observe_failure,
        );
        self.destroy_and_publish(replacement);
    }

    fn destroy_and_publish(&mut self, replacement: LinearVector<T>) {
        self.abandoned = Some(ManuallyDrop::new(replacement));
        let original = self.take_live();
        self.current = Some(GrowthStorage::Interrupted(InterruptedGrowth::new(
            original.elements,
        )));
        self.interrupted_mut().destroy_original();
        self.publish_replacement();
    }

    fn construct_replacement(
        &mut self,
        minimum: usize,
        mut relocate: impl FnMut(&mut T) -> T,
        #[cfg(test)] observe_failure: impl FnOnce(&mut LinearVector<T>),
    ) -> LinearVector<T> {
        let original = self.live_mut();
        let capacity =
            next_capacity_for(original.capacity(), minimum, LinearVector::<T>::max_size());
        LinearVector::construct_owned(
            capacity,
            |destination| original.relocate_unpublished_into(destination, &mut relocate),
            #[cfg(test)]
            observe_failure,
        )
    }

    fn publish_replacement(&mut self) {
        self.take_interrupted().release_empty();
        let replacement = self
            .abandoned
            .take()
            .expect("growth replacement count 0: expected one complete allocation");
        self.current = Some(GrowthStorage::Live(ManuallyDrop::into_inner(replacement)));
    }

    fn live_mut(&mut self) -> &mut LinearVector<T> {
        match self
            .current
            .as_mut()
            .expect("growth ownership: expected current storage")
        {
            GrowthStorage::Live(vector) => vector,
            GrowthStorage::Interrupted(storage) => panic!(
                "growth dead slots {:?}: expected fully repaired storage",
                storage.progress().dead_slots
            ),
        }
    }

    fn take_live(&mut self) -> LinearVector<T> {
        match self
            .current
            .take()
            .expect("growth ownership: expected current storage")
        {
            GrowthStorage::Live(vector) => vector,
            GrowthStorage::Interrupted(_) => {
                unreachable!("growth storage interrupted: expected live owner")
            }
        }
    }

    fn interrupted_mut(&mut self) -> &mut InterruptedGrowth<T> {
        match self
            .current
            .as_mut()
            .expect("growth ownership: expected current storage")
        {
            GrowthStorage::Interrupted(storage) => storage,
            GrowthStorage::Live(_) => {
                unreachable!("growth storage live: expected suspended raw allocation")
            }
        }
    }

    fn take_interrupted(&mut self) -> InterruptedGrowth<T> {
        match self
            .current
            .take()
            .expect("growth ownership: expected current storage")
        {
            GrowthStorage::Interrupted(storage) => storage,
            GrowthStorage::Live(_) => {
                unreachable!("growth storage live: expected suspended raw allocation")
            }
        }
    }

    #[cfg(test)]
    fn take_abandoned_for_test(&mut self) -> Option<LinearVector<T>> {
        self.abandoned.take().map(ManuallyDrop::into_inner)
    }
}

#[cfg(test)]
mod tests;
