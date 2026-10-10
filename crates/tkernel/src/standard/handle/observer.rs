//! Keep callback targets addressable without fabricating an OCCT owning reference.
use super::{Handle, RuntimeTyped};
use std::{marker::PhantomData, ops::Deref};
/// A storage-backed const handle reference; `let observed=handle.observe(); assert_eq!(observed.ref_count(),handle.ref_count());`.
///
/// This observer does not change the embedded OCCT reference count. Its private Arc keeps Rust
/// storage valid if a callback invalidates the originating container; native dangling-reference
/// behavior remains outside the valid API domain. Clone the dereferenced handle to acquire ownership.
pub struct HandleObserver<T: ?Sized + RuntimeTyped> {
    handle: Handle<T>,
}
impl<T: ?Sized + RuntimeTyped> Handle<T> {
    /// Release a container guard while retaining a const handle reference; `let observed=handle.observe(); callback(&observed);`.
    pub fn observe(&self) -> HandleObserver<T> {
        HandleObserver {
            handle: Handle {
                owner: self.owner.clone(),
                projected: PhantomData,
            },
        }
    }
}
impl<T: ?Sized + RuntimeTyped> Deref for HandleObserver<T> {
    type Target = Handle<T>;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
impl<T: ?Sized + RuntimeTyped> Drop for HandleObserver<T> {
    fn drop(&mut self) {
        // Only storage was cloned; remove it before Handle::drop could decrement the native count.
        self.handle.owner.take();
    }
}
