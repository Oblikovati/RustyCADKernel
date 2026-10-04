use super::{Handle, Owner, RuntimeTyped, Transient};
use std::marker::PhantomData;
/// A const handle reference projected to a base type, with no extra owning reference.
pub struct HandleView<'a, T: ?Sized + RuntimeTyped> {
    owner: Option<&'a Owner>,
    projected: PhantomData<T>,
}
impl<'a, T: ?Sized + RuntimeTyped> HandleView<'a, T> {
    pub(super) fn new(owner: Option<&'a Owner>) -> Self {
        Self {
            owner: owner.filter(|owner| T::project_from(owner.get()).is_some()),
            projected: PhantomData,
        }
    }
    /// The viewed owner, for building a new owning handle from this view.
    pub(super) fn owner(&self) -> Option<&'a Owner> {
        self.owner
    }
    /// Check a borrowed null handle; `view.is_null()`.
    pub fn is_null(&self) -> bool {
        self.owner.is_none()
    }
    /// Borrow the projected subobject or interface view; `view.get()`.
    pub fn get(&self) -> Option<&T> {
        T::project_from(self.owner?.get())
    }
}
impl<T: RuntimeTyped> HandleView<'_, T> {
    /// Observe the projected address; `view.as_ptr()`.
    pub fn as_ptr(&self) -> *const T {
        self.get().map_or(std::ptr::null(), std::ptr::from_ref)
    }
}
impl<'a, T: ?Sized + RuntimeTyped> HandleView<'a, T> {
    /// Borrow the viewed object as a transient, as a `const Standard_Transient*`;
    /// `occ::is_kind::<Base>(view.transient())`.
    pub fn transient(&self) -> Option<&'a dyn Transient> {
        self.owner.map(|owner| owner.get())
    }
}
impl<T: ?Sized + RuntimeTyped, U: ?Sized + RuntimeTyped> PartialEq<Handle<U>>
    for HandleView<'_, T>
{
    fn eq(&self, other: &Handle<U>) -> bool {
        match (self.owner, other.owner.as_ref()) {
            (Some(left), Some(right)) => left.address() == right.address(),
            (None, None) => true,
            _ => false,
        }
    }
}
