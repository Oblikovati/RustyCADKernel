//! Capture one actual returned field without retaining its transient owner or target.
use super::{Handle, HandleObserver, HandleReference, RuntimeTyped, SharedState};
use std::sync::{Arc, Weak};

/// Nonowning alias of an actual shared field; `WeakHandleFieldRef::new(&fields,select_camera)`.
/// The selector identifies the captured field once; alias reads never repeat a virtual getter.
pub struct WeakHandleFieldRef<S, T: ?Sized + RuntimeTyped> {
    fields: Weak<SharedState<S>>,
    select: fn(&S) -> &Handle<T>,
    slot_address: usize,
}
impl<S, T: ?Sized + RuntimeTyped> Clone for WeakHandleFieldRef<S, T> {
    fn clone(&self) -> Self {
        Self {
            fields: self.fields.clone(),
            select: self.select,
            slot_address: self.slot_address,
        }
    }
}
impl<S, T: ?Sized + RuntimeTyped> WeakHandleFieldRef<S, T> {
    /// Capture the original field storage without an owner; `Self::new(&fields,select_camera)`.
    /// The locator must always return this same retained field, not repeat a virtual mode selection.
    pub fn new(fields: &Arc<SharedState<S>>, select: fn(&S) -> &Handle<T>) -> Self {
        let slot_address = std::ptr::from_ref(select(&fields.read())).cast::<()>() as usize;
        Self {
            fields: Arc::downgrade(fields),
            select,
            slot_address,
        }
    }
    /// Read current storage without native ownership; `alias.observe().object_address()`.
    pub fn observe(&self) -> HandleObserver<T> {
        // The locator reads the captured field; it is not an object virtual getter.
        self.inspect_field(|| Handle::<T>::default().observe(), Handle::observe)
    }
    /// Read nullness, including safe expiration outside native lifetime; `alias.is_null()`.
    pub fn is_null(&self) -> bool {
        // Expired owner storage is outside the valid native reference lifetime.
        self.inspect_field(|| true, Handle::is_null)
    }
    /// Read the current pointer without a target copy; `alias.object_address()`.
    pub fn object_address(&self) -> *const () {
        // Pointer inspection cannot invent a temporary owning conversion.
        self.inspect_field(std::ptr::null, Handle::object_address)
    }
    /// Identify the original handle field despite target replacement; `alias.reference_address()`.
    pub fn reference_address(&self) -> *const () {
        // Identity refers to the selected native field rather than this Rust wrapper.
        self.inspect_field(std::ptr::null, |slot| std::ptr::from_ref(slot).cast())
    }
    /// Explicitly copy native target ownership; `let owned=alias.clone_handle()`.
    pub fn clone_handle(&self) -> Handle<T> {
        // Only this explicitly owning convenience increments the native target counter.
        self.inspect_field(Handle::default, Handle::clone)
    }
    fn inspect_field<R>(
        &self,
        expired: impl FnOnce() -> R,
        inspect: impl FnOnce(&Handle<T>) -> R,
    ) -> R {
        let Some(storage) = self.fields.upgrade() else {
            return expired();
        };
        let fields = storage.read();
        let slot = (self.select)(&fields);
        self.require_captured_slot(slot);
        inspect(slot)
    }
    fn require_captured_slot(&self, slot: &Handle<T>) {
        let observed = std::ptr::from_ref(slot).cast::<()>() as usize;
        if observed == self.slot_address {
            return;
        }
        super::raise_native_failure::<()>(Err(super::exceptions::program_error(&format!(
            "field locator returned slot={observed:#x}; expected captured slot={:#x}",
            self.slot_address
        ))));
    }
}
impl<S, T: ?Sized + RuntimeTyped> HandleReference<T> for WeakHandleFieldRef<S, T> {
    fn observe(&self) -> HandleObserver<T> {
        // The source field is reread; no owning handle spans a native callback.
        WeakHandleFieldRef::observe(self)
    }
    fn reference_address(&self) -> *const () {
        // Multiple captured aliases identify the actual field rather than their wrapper boxes.
        WeakHandleFieldRef::reference_address(self)
    }
    fn object_address(&self) -> *const () {
        // Identity inspection does not require projecting a retiring target.
        WeakHandleFieldRef::object_address(self)
    }
    fn is_null(&self) -> bool {
        // Expired storage has no safe native pointee to project.
        WeakHandleFieldRef::is_null(self)
    }
}
#[cfg(test)]
#[path = "weak_handle_field_ref_tests.rs"]
mod tests;
