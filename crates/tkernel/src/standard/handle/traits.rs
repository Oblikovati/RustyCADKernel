//! Operators of `opencascade::handle`: `->`/`*` (Deref), `==`/`!=`, `<` and `std::hash`.
use super::{Handle, RuntimeTyped};
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Deref,
};
impl<T: ?Sized + RuntimeTyped> Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
            .expect("Null handle; expected a live object for dereference")
    }
}
impl<T: ?Sized + RuntimeTyped, U: ?Sized + RuntimeTyped> PartialEq<Handle<U>> for Handle<T> {
    fn eq(&self, other: &Handle<U>) -> bool {
        match (&self.owner, &other.owner) {
            (Some(left), Some(right)) => left.address() == right.address(),
            (None, None) => true,
            _ => false,
        }
    }
}
impl<T: ?Sized + RuntimeTyped> Eq for Handle<T> {}
/// Formats handle identity without borrowing object contents; `format!("{handle:?}")`.
impl<T: ?Sized + RuntimeTyped> std::fmt::Debug for Handle<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Handle")
            .field("type", &std::any::type_name::<T>())
            .field("identity", &self.object_address())
            .field("is_null", &self.is_null())
            .finish()
    }
}
impl<T: RuntimeTyped> PartialEq<*const T> for Handle<T> {
    fn eq(&self, other: &*const T) -> bool {
        self.as_ptr() == *other
    }
}
impl<T: RuntimeTyped> PartialEq<Handle<T>> for *const T {
    fn eq(&self, other: &Handle<T>) -> bool {
        *self == other.as_ptr()
    }
}
/// `operator<`: orders by object address, null first, as C++ compares `get()` pointers.
impl<T: ?Sized + RuntimeTyped, U: ?Sized + RuntimeTyped> PartialOrd<Handle<U>> for Handle<T> {
    fn partial_cmp(&self, other: &Handle<U>) -> Option<Ordering> {
        Some(self.address().cmp(&other.address()))
    }
}
impl<T: ?Sized + RuntimeTyped> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address().cmp(&other.address())
    }
}
/// `std::hash<Handle(T)>`: hashes the object address that [`Handle::hash_code`] returns.
impl<T: ?Sized + RuntimeTyped> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address().hash(state);
    }
}
/// Native default-map policy uses the static class specialization; Rust operators retain identity.
impl<T: ?Sized + RuntimeTyped> crate::ncollection::HashKey for Handle<T> {
    fn occt_hash(&self) -> usize {
        T::handle_key_hash(self)
    }
    fn occt_equal(&self, other: &Self) -> bool {
        T::handle_key_equal(self, other)
    }
}
