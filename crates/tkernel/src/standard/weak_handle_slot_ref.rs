//! Weak observations of native handle slots permit owner mutation while aliases remain alive.
use super::{Handle, HandleObserver, RuntimeTyped, SharedState};
use std::sync::{Arc, Weak};
/// Observe a real shared slot without retaining its target or owner; `WeakHandleSlotRef::new(&slot)`.
pub struct WeakHandleSlotRef<T: ?Sized + RuntimeTyped> {
    slot: Weak<SharedState<Handle<T>>>,
}
impl<T: ?Sized + RuntimeTyped> Clone for WeakHandleSlotRef<T> {
    fn clone(&self) -> Self {
        // Native reference aliases do not increment the animation reference counter.
        Self {
            slot: self.slot.clone(),
        }
    }
}
impl<T: ?Sized + RuntimeTyped> WeakHandleSlotRef<T> {
    /// Observe existing storage; `WeakHandleSlotRef::new(&slot)`.
    pub fn new(slot: &Arc<SharedState<Handle<T>>>) -> Self {
        // The owner retains storage; observers must not keep an animation alive.
        Self {
            slot: Arc::downgrade(slot),
        }
    }
    /// Observe null, or safe expiration beyond the native valid-reference lifetime; `alias.is_null()`.
    pub fn is_null(&self) -> bool {
        let slot = self.slot.upgrade();
        slot.is_none_or(|slot| slot.read().is_null())
    }
    /// Observe target identity without retaining it; `alias.object_address()`.
    pub fn object_address(&self) -> *const () {
        self.slot
            .upgrade()
            .map_or(std::ptr::null(), |slot| slot.read().object_address())
    }
    /// Compare storage identity independently of current target; `left.is_same_reference(&right)`.
    pub fn is_same_reference(&self, other: &Self) -> bool {
        // Slot identity survives assignment and does not acquire native ownership.
        self.slot.ptr_eq(&other.slot)
    }
    /// Explicitly retain the current target, or null after owner disposal; `alias.clone_handle()`.
    pub fn clone_handle(&self) -> Handle<T> {
        self.slot
            .upgrade()
            .map_or_else(Handle::default, |slot| slot.read().clone())
    }
    /// Observe current object storage without native owning references; `alias.observe().get()`.
    pub fn observe(&self) -> HandleObserver<T> {
        self.slot.upgrade().map_or_else(
            || Handle::<T>::default().observe(),
            |slot| slot.read().observe(),
        )
    }
    /// Run outside the slot lock; `alias.with_object(|object|object.is_some())`.
    pub fn with_object<R>(&self, inspect: impl FnOnce(Option<&T>) -> R) -> R {
        let observed = self.observe();
        inspect(observed.get())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::standard::StandardTransient;
    #[test]
    fn weak_alias_follows_native_slot_replacement_without_owning_target_or_storage() {
        let first = Handle::new(StandardTransient::default());
        let second = Handle::new(StandardTransient::default());
        let slot = Arc::new(SharedState::new(first.clone()));
        let alias = WeakHandleSlotRef::new(&slot);
        assert_eq!(first.ref_count(), 2);
        assert!(alias.is_same_reference(&alias.clone()));
        assert_eq!(alias.object_address(), first.object_address());
        *slot.write() = second.clone();
        assert_eq!(alias.object_address(), second.object_address());
        assert_weak_storage_observer(&alias, &second);
        alias.with_object(|target| {
            assert!(target.is_some());
            *slot.write() = Handle::default();
        });
        assert!(alias.is_null());
        *slot.write() = second.clone();
        assert_eq!(alias.clone_handle(), second);
        assert_weak_alias_expiration(slot, &alias);
    }
    fn assert_weak_alias_expiration(
        slot: Arc<SharedState<Handle<StandardTransient>>>,
        alias: &WeakHandleSlotRef<StandardTransient>,
    ) {
        drop(slot);
        assert!(alias.is_null());
        assert!(alias.clone_handle().is_null());
    }
    fn assert_weak_storage_observer(
        alias: &WeakHandleSlotRef<StandardTransient>,
        target: &Handle<StandardTransient>,
    ) {
        let observed = alias.observe();
        assert_eq!(target.ref_count(), 2);
        drop(observed);
    }
}
