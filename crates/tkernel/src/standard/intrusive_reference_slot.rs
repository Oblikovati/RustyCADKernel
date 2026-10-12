//! Native handle fields keep their pointer visible while the prior owner is destroyed.
use super::{
    Failure, Handle, HandleObserver, HandleReference, RuntimeTyped, SharedState, Transient,
    WeakHandle,
};
struct ReferenceSlotMembers<T: ?Sized + RuntimeTyped> {
    pointer: WeakHandle<T>,
    ownership: Option<Handle<T>>,
    retiring: bool,
}
/// Actual pointer slot plus its Rust ownership lease; `slot.assign(&replacement)?`.
/// The pointer, rather than an owning snapshot, is the authoritative borrowed field value.
pub struct IntrusiveReferenceSlot<T: ?Sized + RuntimeTyped> {
    members: SharedState<ReferenceSlotMembers<T>>,
}
impl<T: ?Sized + RuntimeTyped> Default for IntrusiveReferenceSlot<T> {
    fn default() -> Self {
        Self {
            members: SharedState::new(ReferenceSlotMembers {
                pointer: WeakHandle::default(),
                ownership: None,
                retiring: false,
            }),
        }
    }
}
impl<T: ?Sized + RuntimeTyped> IntrusiveReferenceSlot<T> {
    /// Preserve native EndScope/BeginScope sequencing; `slot.assign(&source)?`.
    pub fn assign(&self, source: &dyn HandleReference<T>) -> Result<(), Failure> {
        let pointer = source.object_address();
        if self.object_address() == pointer {
            return Ok(());
        }
        // Assign captures its pointer argument before EndScope, not a native owning copy.
        let incoming = source.observe();
        let old = self.begin_retirement(pointer)?;
        drop(old);
        self.finish_retirement(incoming);
        Ok(())
    }
    /// Release ownership without prematurely clearing the pointer; `slot.clear()?`.
    pub fn clear(&self) -> Result<(), Failure> {
        let null = Handle::<T>::default();
        self.assign(&null)
    }
    fn begin_retirement(&self, incoming: *const ()) -> Result<Option<Handle<T>>, Failure> {
        let mut members = self.members.write();
        if members.retiring {
            return Err(super::exceptions::program_error(&format!(
                "incoming={incoming:p}, retiring={:p}; expected idle slot or same target",
                members.pointer.object_address()
            )));
        }
        members.retiring = true;
        Ok(members.ownership.take())
    }
    fn finish_retirement(&self, incoming: HandleObserver<T>) {
        // The last old destructor has returned before native ownership is acquired.
        // OCCT 3d097a0 EndScope clears a callback-replaced pointer without releasing its reference.
        self.clear_end_scope_pointer();
        let ownership = incoming.clone();
        let pointer = ownership.downgrade();
        let mut members = self.members.write();
        members.pointer = pointer;
        members.ownership = Some(ownership);
        members.retiring = false;
    }
    fn clear_end_scope_pointer(&self) {
        let mut members = self.members.write();
        if let Some(displaced) = members.ownership.take() {
            // Native loses this reference at every EndScope boundary, including nested destructors.
            std::mem::forget(displaced);
        }
        members.pointer = WeakHandle::default();
    }
    /// Observe pointer nullness even during a destructor; `slot.is_null()`.
    pub fn is_null(&self) -> bool {
        let members = self.members.read();
        members.pointer.is_null()
    }
    /// Read the actual pointer field, including its retiring phase; `slot.object_address()`.
    pub fn object_address(&self) -> *const () {
        let members = self.members.read();
        members.pointer.object_address()
    }
    /// Borrow current physical storage without intrusive ownership; `slot.observe()`.
    /// A retiring pointee cannot be safely projected through an expired Arc; no live facade is invented.
    pub fn observe(&self) -> HandleObserver<T> {
        let members = self.members.read();
        let observed = members.pointer.observe();
        let pointer = members.pointer.object_address();
        drop(members);
        if !pointer.is_null() && observed.ref_count() == 0 {
            super::raise_native_failure::<()>(Err(super::exceptions::null_object(&format!(
                "pointer={:p}, native count=0; expected a projectable live pointee",
                pointer
            ))));
        }
        observed
    }
    /// Reborrow the actual destructor pointee; `slot.with_retiring_pointee(self, |p| p.ref_count())?`.
    /// The closure cannot return a reference borrowed from its pointee argument.
    /// ```compile_fail
    /// use rustycad_tkernel::standard::{IntrusiveReferenceSlot, StandardTransient};
    /// let slot = IntrusiveReferenceSlot::<StandardTransient>::default();
    /// let retiring = StandardTransient::default();
    /// let escaped = slot.with_retiring_pointee(&retiring, |pointee| pointee);
    /// ```
    pub fn with_retiring_pointee<R>(
        &self,
        retiring: &dyn Transient,
        inspect: impl FnOnce(&T) -> R,
    ) -> Result<R, Failure> {
        self.require_retiring_pointer(retiring)?;
        let pointee = T::project_from(retiring).ok_or_else(|| {
            super::exceptions::type_mismatch(&format!(
                "type={}; expected retiring {} projection",
                retiring.dynamic_type().name(),
                T::TYPE_NAME
            ))
        })?;
        Ok(inspect(pointee))
    }
    /// Lend native mutation scope without escaping the destructor borrow; `slot.with_retiring_reference(self, |r| r.clear())?`.
    /// ```compile_fail
    /// use rustycad_tkernel::standard::{IntrusiveReferenceSlot, StandardTransient};
    /// let slot = IntrusiveReferenceSlot::<StandardTransient>::default();
    /// let retiring = StandardTransient::default();
    /// let escaped = slot.with_retiring_reference(&retiring, |scope| scope);
    /// ```
    pub fn with_retiring_reference<R>(
        &self,
        retiring: &dyn Transient,
        inspect: impl FnOnce(IntrusiveRetirementBorrow<'_, T>) -> R,
    ) -> Result<R, Failure> {
        self.require_retiring_pointer(retiring)?;
        Ok(inspect(IntrusiveRetirementBorrow {
            slot: self,
            retiring,
        }))
    }
    fn require_retiring_pointer(&self, retiring: &dyn Transient) -> Result<(), Failure> {
        let members = self.members.read();
        let incoming = std::ptr::from_ref(retiring).cast::<()>();
        let expected = members.pointer.object_address();
        if !members.retiring || incoming != expected {
            return Err(super::exceptions::program_error(&format!(
                "pointer={incoming:p}, retiring={}; expected current retiring pointer={expected:p}",
                members.retiring
            )));
        }
        Ok(())
    }
    /// Inspect the native count without keeping a field guard; `slot.ref_count()`.
    pub fn ref_count(&self) -> i32 {
        let members = self.members.read();
        members.pointer.observe().ref_count()
    }
    /// Identify the actual pointer member for retained aliases; `slot.reference_address()`.
    pub fn reference_address(&self) -> *const () {
        let members = self.members.read();
        std::ptr::from_ref(&members.pointer).cast()
    }
}
#[cfg(test)]
#[path = "intrusive_reference_slot_tests.rs"]
mod tests;
impl<T: ?Sized + RuntimeTyped> HandleReference<T> for IntrusiveReferenceSlot<T> {
    fn observe(&self) -> HandleObserver<T> {
        let slot = self;
        IntrusiveReferenceSlot::observe(slot)
    }
    fn reference_address(&self) -> *const () {
        let slot = self;
        IntrusiveReferenceSlot::reference_address(slot)
    }
    fn object_address(&self) -> *const () {
        let slot = self;
        IntrusiveReferenceSlot::object_address(slot)
    }
    fn is_null(&self) -> bool {
        let slot = self;
        IntrusiveReferenceSlot::is_null(slot)
    }
}

/// A real destructor reborrow plus its original field; `borrow.assign(&incoming)?`.
/// The scope carries no target ownership and must not escape its constructor closure.
pub struct IntrusiveRetirementBorrow<'a, T: ?Sized + RuntimeTyped> {
    slot: &'a IntrusiveReferenceSlot<T>,
    retiring: &'a dyn Transient,
}
impl<T: ?Sized + RuntimeTyped> IntrusiveRetirementBorrow<'_, T> {
    /// Native nested Assign, including count zero to negative one; `borrow.assign(&replacement)?`.
    pub fn assign(&self, source: &dyn HandleReference<T>) -> Result<(), Failure> {
        let pointer = source.object_address();
        if self.slot.object_address() == pointer {
            return Ok(());
        }
        let incoming = source.observe();
        self.release_current_reference();
        let owner = incoming.clone();
        let mut members = self.slot.members.write();
        members.pointer = owner.downgrade();
        members.ownership = Some(owner);
        Ok(())
    }
    /// Native nested Nullify; `borrow.clear()?`.
    pub fn clear(&self) -> Result<(), Failure> {
        let null = Handle::<T>::default();
        self.assign(&null)
    }
    fn release_current_reference(&self) {
        let mut members = self.slot.members.write();
        let is_original =
            members.pointer.object_address() == std::ptr::from_ref(self.retiring).cast();
        let ownership = members.ownership.take();
        drop(members);
        if is_original {
            self.retiring.transient_base().decrement_ref_counter();
        }
        drop(ownership);
        self.slot.clear_end_scope_pointer();
    }
}
