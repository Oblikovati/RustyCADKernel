//! Erase a genuine handle-slot borrow, preserving live replacement and intrusive ownership counts.
use super::{
    Failure, Handle, HandleFieldRef, HandleObserver, RetainedHandleReference, RuntimeTyped,
};
use std::marker::PhantomData;

/// Project a genuine live handle slot into a registered base/interface without owning it.
/// Example: `let base=ProjectedHandleReference::<Concrete,dyn Source>::new(&slot)`.
pub struct ProjectedHandleReference<'a, T: ?Sized + RuntimeTyped, U: ?Sized + RuntimeTyped> {
    source: &'a dyn HandleReference<T>,
    projected: PhantomData<fn() -> U>,
}
impl<'a, T: ?Sized + RuntimeTyped, U: ?Sized + RuntimeTyped> ProjectedHandleReference<'a, T, U> {
    /// Borrow the actual original slot; `ProjectedHandleReference::<Concrete,dyn Source>::new(&slot)`.
    pub fn new(source: &'a dyn HandleReference<T>) -> Self {
        Self {
            source,
            projected: PhantomData,
        }
    }
}
impl<T: ?Sized + RuntimeTyped, U: ?Sized + RuntimeTyped> HandleReference<U>
    for ProjectedHandleReference<'_, T, U>
{
    fn retain_reference(&self) -> Result<RetainedHandleReference<U>, Failure> {
        self.source
            .retain_reference()
            .map(RetainedHandleReference::project)
    }
    fn observe(&self) -> HandleObserver<U> {
        // Reread on every use: a callback may have replaced the original caller's slot.
        self.source.observe().observe_as::<U>()
    }
    fn reference_address(&self) -> *const () {
        self.source.reference_address()
    }
    fn object_address(&self) -> *const () {
        // OCCT 3d097a0 Standard_Handle.hxx:122 reads the pointer without RTTI or pointee access.
        self.source.object_address()
    }
    fn is_null(&self) -> bool {
        // OCCT 3d097a0 Standard_Handle.hxx:94 tests entity, even after virtual Delete fails.
        self.source.is_null()
    }
}
/// Native constHandle reference contract for local slots and retained fields.
/// Example: `let observed=reference.observe(); assert_eq!(reference.object_address(),observed.object_address())`.
pub trait HandleReference<T: ?Sized + RuntimeTyped> {
    /// Retain a supported original slot, never its target; `let alias=slot.retain_reference()?`.
    fn retain_reference(&self) -> Result<RetainedHandleReference<T>, Failure> {
        Err(Failure::with_message(
            "Standard_ProgramError",
            &format!(
                "handle slot {:p} has borrowed lifetime; expected an explicit retained-slot capability",
                self.reference_address()
            ),
        ))
    }
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
    fn object_address(&self) -> *const () {
        Handle::object_address(self)
    }
    fn is_null(&self) -> bool {
        Handle::is_null(self)
    }
}
impl<S, T: ?Sized + RuntimeTyped> HandleReference<T> for HandleFieldRef<'_, S, T> {
    fn observe(&self) -> HandleObserver<T> {
        HandleFieldRef::observe(self)
    }
    fn reference_address(&self) -> *const () {
        HandleFieldRef::reference_address(self)
    }
    fn object_address(&self) -> *const () {
        HandleFieldRef::object_address(self)
    }
    fn is_null(&self) -> bool {
        HandleFieldRef::is_null(self)
    }
}
#[cfg(test)]
#[path = "handle_reference_tests.rs"]
mod tests;
