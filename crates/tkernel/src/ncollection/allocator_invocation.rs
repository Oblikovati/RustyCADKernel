//! Native virtual calls carry weak identity and immutable code, never a receiver borrow through callbacks.
#![allow(unsafe_code)] // Explicit pointer-release and callback-free receiver-capture contracts are tested.
use super::{AllocatorDispatchCode, AllocatorSource};
use crate::standard::{Failure, Handle, HandleObserver, RuntimeTyped, WeakHandle};
use std::ptr::NonNull;
/// A reference-free dynamic invocation; `invocation.allocate(32)`.
/// Field capture finishes before callbacks; subsequent captures reread actual surviving storage.
pub struct AllocatorInvocation {
    source: WeakHandle<dyn AllocatorSource>,
    code: AllocatorDispatchCode,
}
impl AllocatorInvocation {
    /// Capture original dynamic code without intrusive ownership; `AllocatorInvocation::from_handle(&owner)?.allocate(32)`.
    pub fn from_handle<T: ?Sized + RuntimeTyped + AllocatorSource>(
        source: &Handle<T>,
    ) -> Result<Self, Failure> {
        let observed = source.observe_as::<dyn AllocatorSource>();
        if observed.is_null() {
            return Err(crate::standard::exceptions::null_object(&format!(
                "allocator pointer={:p}; expected live registered AllocatorSource",
                source.object_address()
            )));
        }
        Ok(Self::from_observer(&observed))
    }
    pub(super) fn from_observer(source: &HandleObserver<dyn AllocatorSource>) -> Self {
        // Only immutable code and weak identity cross the virtual-call boundary.
        Self {
            source: source.downgrade(),
            code: source.dispatch_code(),
        }
    }
    /// Briefly access actual fields, returning native locals; `invocation.capture::<MyAllocator,_>(capture_operands)?`.
    /// # Safety
    /// Capture must not invoke callbacks, retire native resources, or return a receiver borrow.
    /// Later native field uses require a fresh capture; captured reference fields must retain identity.
    ///
    /// ```compile_fail
    /// use rustycad_tkernel::{ncollection::{AllocatorInvocation, AllocatorObject}, standard::Handle};
    /// let owner = Handle::new(AllocatorObject::default());
    /// let invocation = AllocatorInvocation::from_handle(&owner).unwrap();
    /// let escaped = unsafe { invocation.capture::<AllocatorObject, _>(|receiver| receiver) };
    /// ```
    pub unsafe fn capture<T: ?Sized + RuntimeTyped, R: 'static>(
        &self,
        capture: fn(&T) -> R,
    ) -> Result<R, Failure> {
        let observed = self.source.observe().observe_as::<T>();
        let target = observed.get().ok_or_else(|| self.expired_capture::<T>())?;
        Ok(capture(target))
    }
    fn ensure_live(&self) -> Result<(), Failure> {
        if self.source.observe().is_null() {
            return Err(self.expired_capture::<dyn AllocatorSource>());
        }
        Ok(())
    }
    fn expired_capture<T: ?Sized + RuntimeTyped>(&self) -> Failure {
        crate::standard::exceptions::null_object(&format!(
            "allocator pointer={:p}, projection={}; expected live actual dispatch storage",
            self.source.object_address(),
            T::TYPE_NAME
        ))
    }
    /// Capture only independent parent backing; `let policy=invocation.parent_policy()?`.
    pub fn parent_policy(&self) -> Result<super::AllocatorPolicyInvocation, Failure> {
        // SAFETY: the selector reads shared physical backing and cannot call or retire native objects.
        unsafe { self.capture(super::allocator_default_dispatch::default_policy) }
    }
    /// Execute qualified parent Allocate on actual backing; `invocation.allocate_base(32)`.
    pub fn allocate_base(&self, size: usize) -> Option<NonNull<u8>> {
        let policy = crate::standard::raise_native_failure(self.parent_policy());
        policy.allocate(size, true)
    }
    /// Execute qualified parent Optimal; `invocation.allocate_optimal_base(32)`.
    pub fn allocate_optimal_base(&self, size: usize) -> Option<NonNull<u8>> {
        let policy = crate::standard::raise_native_failure(self.parent_policy());
        policy.allocate_optimal(size)
    }
    /// Execute qualified nullable parent Free; `unsafe { invocation.free_base(pointer) }?`.
    /// # Safety
    /// Non-null pointer must be accepted by the actual parent's live physical policy.
    pub unsafe fn free_base(&self, pointer: *mut u8) -> Result<(), Failure> {
        let policy = self.parent_policy()?;
        // SAFETY: the caller supplies the parent's accepted original pointer.
        unsafe { policy.free_nullable(pointer) }
    }
    /// Invoke Allocate without retaining the logical receiver; `invocation.allocate(32)`.
    pub fn allocate(&self, size: usize) -> Option<NonNull<u8>> {
        crate::standard::raise_native_failure(self.ensure_live());
        (self.code.allocate)(self, size)
    }
    /// Invoke AllocateOptimal with the original code table; `invocation.allocate_optimal(32)`.
    pub fn allocate_optimal(&self, size: usize) -> Option<NonNull<u8>> {
        crate::standard::raise_native_failure(self.ensure_live());
        (self.code.allocate_optimal)(self, size)
    }
    /// Invoke nullable Free without a receiver borrow; `unsafe { invocation.free(pointer) }?`.
    /// # Safety
    /// Non-null pointers must remain live and be accepted by this actual allocator's virtual body.
    pub unsafe fn free(&self, pointer: *mut u8) -> Result<(), Failure> {
        self.ensure_live()?;
        // SAFETY: the caller supplies this body's accepted original allocation pointer.
        unsafe { (self.code.free)(self, pointer) }
    }
}
