//! Only MaybeUninit storage owns bytes while the native registered range contains holes.
#![allow(unsafe_code)]
use super::GrowthProgress;
use crate::ncollection::raw_element_lifetime::{construct_in_slot, destroy_in_slot};
use std::mem::{ManuallyDrop, MaybeUninit};

pub(super) struct InterruptedGrowth<T> {
    slots: ManuallyDrop<Vec<MaybeUninit<T>>>,
    destroyed: usize,
    restored: usize,
}
impl<T> InterruptedGrowth<T> {
    pub(super) fn new(elements: Vec<T>) -> Self {
        let mut original = ManuallyDrop::new(elements);
        // SAFETY: identical layout/alignment and unchanged allocation provenance/capacity.
        // MaybeUninit allows later ended lifetimes without a partially dead Vec<T> owner.
        let slots = unsafe {
            Vec::from_raw_parts(
                original.as_mut_ptr().cast(),
                original.len(),
                original.capacity(),
            )
        };
        Self {
            slots: ManuallyDrop::new(slots),
            destroyed: 0,
            restored: 0,
        }
    }

    #[cfg(test)]
    pub(super) fn raw_original_for_test(&mut self) -> (*mut T, usize, usize) {
        (
            self.slots.as_mut_ptr().cast(),
            self.slots.len(),
            self.slots.capacity(),
        )
    }

    pub(super) fn progress(&self) -> GrowthProgress {
        GrowthProgress {
            size: self.slots.len(),
            capacity: self.slots.capacity(),
            dead_slots: self.restored..self.destroyed,
        }
    }

    pub(super) fn original_ptr(&self) -> *const T {
        self.slots.as_ptr().cast()
    }

    pub(super) fn destroy_original(&mut self) {
        while self.destroyed < self.slots.len() {
            let index = self.destroyed;
            self.destroyed += 1;
            // SAFETY: this original slot is live and exclusive. Its lifetime ends when
            // destruction begins, including on unwind; metadata advances before the call.
            unsafe { destroy_in_slot(self.slots.as_mut_ptr().add(index).cast::<T>()) };
        }
    }

    pub(super) fn repair_with(&mut self, mut construct: impl FnMut(usize) -> T) {
        while self.restored < self.destroyed {
            let index = self.restored;
            // SAFETY: the tracked dead range has no live T. Failed construction writes
            // nothing; only a successful write advances the repaired lifetime boundary.
            unsafe {
                construct_in_slot(self.slots.as_mut_ptr().add(index).cast(), || {
                    construct(index)
                })
            };
            self.restored += 1;
        }
    }

    pub(super) fn into_initialized(mut self) -> Vec<T> {
        assert_eq!(
            self.restored, self.destroyed,
            "growth repaired prefix: expected no remaining holes"
        );
        // SAFETY: every original registered slot is live again and retains its allocation.
        // The ManuallyDrop byte owner relinquishes ownership without freeing it.
        unsafe {
            Vec::from_raw_parts(
                self.slots.as_mut_ptr().cast(),
                self.slots.len(),
                self.slots.capacity(),
            )
        }
    }

    pub(super) fn release_empty(self) {
        assert_eq!(
            self.destroyed,
            self.slots.len(),
            "growth destruction count: expected every original slot dead"
        );
        // All registered values are dead. MaybeUninit drops no values while freeing bytes;
        // any native unpublished spare objects likewise receive no implicit destructor.
        drop(ManuallyDrop::into_inner(self.slots));
    }
}
