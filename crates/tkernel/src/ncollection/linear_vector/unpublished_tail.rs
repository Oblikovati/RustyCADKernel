//! OCCT Resize publishes its tail only after every placement constructor succeeds.
#![allow(unsafe_code)]
use super::LinearVector;
use std::mem::MaybeUninit;

impl<T> LinearVector<T> {
    pub(super) fn construct_unpublished_tail(
        &mut self,
        size: usize,
        mut construct: impl FnMut() -> T,
    ) {
        let additional = size - self.len();
        for slot in &mut self.elements.spare_capacity_mut()[..additional] {
            // Native placement construction ends a previous unpublished object's lifetime
            // before its initializer runs, including when that initializer throws.
            *slot = MaybeUninit::uninit();
            slot.write(construct());
        }
        // SAFETY: the owned spare slice bounds every write. All requested tail values
        // are initialized on this success path, and the original prefix remains valid.
        // On unwind, len stays old: native intentionally omits unpublished destructors.
        unsafe { self.elements.set_len(size) };
    }
}
