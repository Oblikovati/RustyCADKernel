//! Stable local ownership for OCCT 3d097a0 AIS_ViewController's aPickedView const-reference scope.
use super::retained_handle_reference::RetainedSlotSource;
use super::{
    Failure, Handle, HandleObserver, HandleReference, RetainedHandleReference, RuntimeTyped,
};
use std::rc::{Rc, Weak};

/// Own one original local slot through callbacks; `let slot=ScopedHandleReferenceSlot::from_owned(picked)`.
/// Its aliases expire on return/unwind. The local value is fixed; this does not erase ordinary stack borrows.
pub struct ScopedHandleReferenceSlot<T: ?Sized + RuntimeTyped> {
    slot: Rc<Handle<T>>,
}
impl<T: ?Sized + RuntimeTyped> ScopedHandleReferenceSlot<T> {
    /// Transfer native ownership without a clone; `let slot=ScopedHandleReferenceSlot::from_owned(picked)`.
    pub fn from_owned(handle: Handle<T>) -> Self {
        Self {
            slot: Rc::new(handle),
        }
    }
}
impl<T: ?Sized + RuntimeTyped> HandleReference<T> for ScopedHandleReferenceSlot<T> {
    fn retain_reference(&self) -> Result<RetainedHandleReference<T>, Failure> {
        Ok(RetainedHandleReference::from_source(ScopedSlotAlias {
            slot: Rc::downgrade(&self.slot),
        }))
    }
    fn observe(&self) -> HandleObserver<T> {
        // Observer leases avoid the intrusive retain that a copied callback argument would add.
        self.slot.observe()
    }
    fn reference_address(&self) -> *const () {
        // Bind the genuine Handle field, not the scope wrapper or its target.
        Rc::as_ptr(&self.slot).cast()
    }
    fn object_address(&self) -> *const () {
        // Read original pointer metadata without projecting a retired target.
        self.slot.object_address()
    }
    fn is_null(&self) -> bool {
        // A live null slot still has a valid original reference identity.
        self.slot.is_null()
    }
}
struct ScopedSlotAlias<T: ?Sized + RuntimeTyped> {
    slot: Weak<Handle<T>>,
}
impl<T: ?Sized + RuntimeTyped> HandleReference<T> for ScopedSlotAlias<T> {
    fn observe(&self) -> HandleObserver<T> {
        self.slot
            .upgrade()
            .map_or_else(|| Handle::<T>::default().observe(), |slot| slot.observe())
    }
    fn reference_address(&self) -> *const () {
        self.slot
            .upgrade()
            .map_or(std::ptr::null(), |slot| Rc::as_ptr(&slot).cast())
    }
    fn object_address(&self) -> *const () {
        self.slot
            .upgrade()
            .map_or(std::ptr::null(), |slot| slot.object_address())
    }
    fn is_null(&self) -> bool {
        // Expiration is Rust safety behavior, not a native dangling-reference observation.
        self.slot.upgrade().is_none_or(|slot| slot.is_null())
    }
}
impl<T: ?Sized + RuntimeTyped> RetainedSlotSource<T> for ScopedSlotAlias<T> {
    fn is_expired(&self) -> bool {
        // Only the scoped owner owns this storage; retained aliases cannot postpone retirement.
        self.slot.strong_count() == 0
    }
}
#[cfg(test)]
#[path = "scoped_handle_reference_tests.rs"]
mod tests;
