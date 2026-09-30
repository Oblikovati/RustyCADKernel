//! A non-owning pointer to a handled object: the Rust form of the raw `void*` or
//! `T*` fields that some OCCT classes keep beside their handles (MAT_Arc, MAT_Node
//! and MAT_BasicElt of TKTopAlgo) so that a graph of transients has no reference
//! cycle.
use super::{Handle, RuntimeTyped, Transient};
use std::{
    marker::PhantomData,
    sync::{Arc, Weak},
};

/// A raw pointer to a transient: it never owns the object, it is null or not, and
/// [`WeakHandle::upgrade`] gives a handle to the object while it is alive
/// (`occ::handle<T>((T*)pointer)`). Where C++ would keep a dangling pointer to a
/// destroyed object, the upgrade gives a null handle instead of undefined behaviour;
/// [`WeakHandle::is_null`] still answers `pointer == nullptr` as C++ does.
///
/// ```
/// use rustycad_tkernel::standard::{Handle, StandardTransient, WeakHandle};
/// let owner: Handle<StandardTransient> = Handle::new(StandardTransient::default());
/// let pointer: WeakHandle<StandardTransient> = owner.downgrade();
/// assert!(!pointer.is_null() && pointer.upgrade() == owner);
/// assert_eq!(owner.ref_count(), 1);
/// assert!(WeakHandle::<StandardTransient>::default().is_null());
/// ```
pub struct WeakHandle<T: ?Sized + RuntimeTyped> {
    owner: Option<Weak<dyn Transient>>,
    projected: PhantomData<T>,
}
impl<T: ?Sized + RuntimeTyped> Default for WeakHandle<T> {
    fn default() -> Self {
        Self {
            owner: None,
            projected: PhantomData,
        }
    }
}
impl<T: ?Sized + RuntimeTyped> Clone for WeakHandle<T> {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            projected: PhantomData,
        }
    }
}
impl<T: ?Sized + RuntimeTyped> WeakHandle<T> {
    /// True for a null pointer, whether or not the pointed object is still alive;
    /// `pointer.is_null()`.
    pub fn is_null(&self) -> bool {
        self.owner.is_none()
    }
    /// A new owning handle to the pointed object, or a null handle when the pointer
    /// is null or the object is gone; `let arc: Handle<T> = pointer.upgrade();`.
    pub fn upgrade(&self) -> Handle<T> {
        let Some(owner) = self.owner.as_ref().and_then(Weak::upgrade) else {
            return Handle::default();
        };
        Handle::adopt(Some(&owner))
    }
}
impl<T: ?Sized + RuntimeTyped> Handle<T> {
    /// The raw pointer of this handle (`handle.get()` stored as a `T*`): it does not
    /// change the reference count; `let pointer = handle.downgrade();`.
    pub fn downgrade(&self) -> WeakHandle<T> {
        WeakHandle {
            owner: self.owner.as_ref().map(Arc::downgrade),
            projected: PhantomData,
        }
    }
}

impl<T: ?Sized + RuntimeTyped> WeakHandle<T> {
    /// Stable non-owning identity, including after destruction; `pointer.object_address() == owner.object_address()`.
    pub fn object_address(&self) -> *const () {
        self.owner
            .as_ref()
            .map_or(std::ptr::null(), |owner| owner.as_ptr() as *const ())
    }
}
impl<T: ?Sized + RuntimeTyped> PartialEq for WeakHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.object_address() == other.object_address()
    }
}
impl<T: ?Sized + RuntimeTyped> Eq for WeakHandle<T> {}
impl<T: ?Sized + RuntimeTyped> std::fmt::Debug for WeakHandle<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("WeakHandle")
            .field(&self.object_address())
            .finish()
    }
}
impl<T: ?Sized + RuntimeTyped> crate::ncollection::HashKey for WeakHandle<T> {
    fn occt_hash(&self) -> usize {
        self.object_address() as usize
    }
}
