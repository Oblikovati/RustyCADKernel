//! Explicit local retention keeps the slot capability, without owning its native target.
use super::{Failure, HandleObserver, HandleReference, RuntimeTyped};
use std::marker::PhantomData;
use std::rc::Rc;

pub(super) trait RetainedSlotSource<T: ?Sized + RuntimeTyped>: HandleReference<T> {
    fn is_expired(&self) -> bool;
}
/// A noncounting, thread-local alias with checked expiration; `let alias=slot.retain_reference()?`.
/// Ordinary stack references do not support lifetime erasure; retaining this alias never extends its owner.
pub struct RetainedHandleReference<T: ?Sized + RuntimeTyped> {
    source: Rc<dyn RetainedSlotSource<T>>,
}
impl<T: ?Sized + RuntimeTyped> RetainedHandleReference<T> {
    pub(super) fn from_source(source: impl RetainedSlotSource<T> + 'static) -> Self {
        Self {
            source: Rc::new(source),
        }
    }
    /// Distinguish expiration from a live null value; `assert!(!alias.is_expired())`.
    pub fn is_expired(&self) -> bool {
        self.source.is_expired()
    }
    pub(super) fn project<U: ?Sized + RuntimeTyped>(self) -> RetainedHandleReference<U> {
        RetainedHandleReference::from_source(RetainedProjection {
            source: self,
            projected: PhantomData,
        })
    }
}
impl<T: ?Sized + RuntimeTyped> Clone for RetainedHandleReference<T> {
    fn clone(&self) -> Self {
        Self {
            source: Rc::clone(&self.source),
        }
    }
}
impl<T: ?Sized + RuntimeTyped> HandleReference<T> for RetainedHandleReference<T> {
    fn retain_reference(&self) -> Result<Self, Failure> {
        Ok(self.clone())
    }
    fn observe(&self) -> HandleObserver<T> {
        self.source.observe()
    }
    fn reference_address(&self) -> *const () {
        self.source.reference_address()
    }
    fn object_address(&self) -> *const () {
        self.source.object_address()
    }
    fn is_null(&self) -> bool {
        self.source.is_null()
    }
}
struct RetainedProjection<T: ?Sized + RuntimeTyped, U: ?Sized + RuntimeTyped> {
    source: RetainedHandleReference<T>,
    projected: PhantomData<fn() -> U>,
}
impl<T: ?Sized + RuntimeTyped, U: ?Sized + RuntimeTyped> HandleReference<U>
    for RetainedProjection<T, U>
{
    fn observe(&self) -> HandleObserver<U> {
        self.source.observe().observe_as::<U>()
    }
    fn reference_address(&self) -> *const () {
        self.source.reference_address()
    }
    fn object_address(&self) -> *const () {
        self.source.object_address()
    }
    fn is_null(&self) -> bool {
        self.source.is_null()
    }
}
impl<T: ?Sized + RuntimeTyped, U: ?Sized + RuntimeTyped> RetainedSlotSource<U>
    for RetainedProjection<T, U>
{
    fn is_expired(&self) -> bool {
        self.source.is_expired()
    }
}
