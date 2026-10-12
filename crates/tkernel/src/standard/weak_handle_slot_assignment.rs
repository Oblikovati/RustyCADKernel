//! Owned callback assignment capability is separate from native read-only handle aliases.
use super::{Failure, Handle, RuntimeTyped, SharedState};
use std::sync::{Arc, Weak};
/// Assign the actual owned slot during callbacks; `assignment.assign(&replacement)?`.
/// This Rust convenience permits native-valid setters without borrowing an entire controller.
pub struct WeakHandleSlotAssignment<T: ?Sized + RuntimeTyped> {
    slot: Weak<SharedState<Handle<T>>>,
}
impl<T: ?Sized + RuntimeTyped> Clone for WeakHandleSlotAssignment<T> {
    fn clone(&self) -> Self {
        // A second capability retains neither target nor owner storage.
        Self {
            slot: self.slot.clone(),
        }
    }
}
impl<T: ?Sized + RuntimeTyped> WeakHandleSlotAssignment<T> {
    /// Connect to actual owner storage; `WeakHandleSlotAssignment::new(&slot)`.
    pub fn new(slot: &Arc<SharedState<Handle<T>>>) -> Self {
        let slot = Arc::downgrade(slot);
        Self { slot }
    }
    /// Perform the same native owning assignment; `assignment.assign(&replacement)?`.
    pub fn assign(&self, target: &Handle<T>) -> Result<(), Failure> {
        let slot = self.slot.upgrade().ok_or_else(|| {
            super::exceptions::null_object(
                "assignment owner=expired; expected live handle-slot owner",
            )
        })?;
        *slot.write() = target.clone();
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::standard::{StandardTransient, WeakHandleSlotRef};
    #[test]
    fn callback_assignment_updates_readonly_alias_and_preserves_native_ownership() {
        let target = Handle::new(StandardTransient::default());
        let slot = Arc::new(SharedState::new(Handle::default()));
        let alias = WeakHandleSlotRef::new(&slot);
        let assignment = WeakHandleSlotAssignment::new(&slot);
        assignment.clone().assign(&target).unwrap();
        assert_eq!(target.ref_count(), 2);
        assert_eq!(alias.object_address(), target.object_address());
        let observed = alias.observe();
        assignment.assign(&Handle::default()).unwrap();
        assert!(alias.is_null());
        assert_eq!(target.ref_count(), 1);
        drop(observed);
        drop(slot);
        assert!(assignment.assign(&target).is_err());
        assert_eq!(target.ref_count(), 1);
    }
}
