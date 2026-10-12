//! Erase a genuine handle-slot borrow, preserving live replacement and intrusive ownership counts.
use super::{Handle, HandleFieldRef, HandleObserver, RuntimeTyped};
/// Native constHandle reference contract for local slots and retained fields.
/// Example: `let observed=reference.observe(); assert_eq!(reference.object_address(),observed.object_address())`.
pub trait HandleReference<T: ?Sized + RuntimeTyped> {
    /// Lease the current target without owning it; `let target=reference.observe()`.
    fn observe(&self) -> HandleObserver<T>;
    /// Identify the actual handle slot, independently of its target; `reference.reference_address()`.
    fn reference_address(&self) -> *const ();
    /// Read target identity after any field replacement; `reference.object_address()`.
    fn object_address(&self) -> *const () {
        self.observe().object_address()
    }
    /// Read current null assignment; `reference.is_null()`.
    fn is_null(&self) -> bool {
        self.observe().is_null()
    }
    /// Explicitly own a target snapshot; `let owned=reference.clone_handle()`.
    fn clone_handle(&self) -> Handle<T> {
        self.observe().clone()
    }
}
impl<T: ?Sized + RuntimeTyped> HandleReference<T> for Handle<T> {
    fn observe(&self) -> HandleObserver<T> {
        Handle::observe(self)
    }
    fn reference_address(&self) -> *const () {
        std::ptr::from_ref(self).cast()
    }
}
impl<S, T: ?Sized + RuntimeTyped> HandleReference<T> for HandleFieldRef<'_, S, T> {
    fn observe(&self) -> HandleObserver<T> {
        HandleFieldRef::observe(self)
    }
    fn reference_address(&self) -> *const () {
        HandleFieldRef::reference_address(self)
    }
}
#[cfg(test)]
#[path = "handle_reference_tests.rs"]
mod tests;
