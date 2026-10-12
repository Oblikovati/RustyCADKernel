//! Native owning allocator handles are separate from noncounting physical allocation leases.
use super::BaseAllocator;
use crate::ncollection::AllocatorSource;
use crate::standard::{Handle, HandleObserver, RuntimeTyped};
pub(super) enum AllocatorReference {
    Policy,
    Owned(Handle<dyn AllocatorSource>),
    Storage(super::AllocatorStorageLease),
}
impl BaseAllocator {
    /// Acquire the SAME derived allocator; `BaseAllocator::from_handle(&allocator)`.
    /// This factory registers only a weak compatibility alias in the genuine embedded base.
    pub fn from_handle<T: ?Sized + RuntimeTyped + AllocatorSource>(handle: &Handle<T>) -> Self {
        let source = handle.cast::<dyn AllocatorSource>();
        assert!(
            !source.is_null(),
            "allocator={:?}; expected registered AllocatorSource interface",
            handle
        );
        let resource = source.allocator_object_base().resource();
        *resource.fields.registration.write() = source.downgrade();
        let mut facade = resource.facade_with_reference(AllocatorReference::Policy);
        facade.fields.reference = AllocatorReference::Owned(source);
        facade
    }
    /// Retain physical storage with zero native ownership; `let lease=allocator.storage_lease()`.
    pub fn storage_lease(&self) -> Self {
        if let AllocatorReference::Storage(backing) = &self.fields.reference {
            return self.facade_with_reference(AllocatorReference::Storage(backing.clone()));
        }
        let backend = self
            .dispatch_source(|source| source.physical_storage())
            .unwrap_or_else(|| self.physical_storage());
        let reference = AllocatorReference::Storage(backend);
        self.facade_with_reference(reference)
    }
    fn facade_with_reference(&self, reference: AllocatorReference) -> Self {
        Self {
            fields: Box::new(super::AllocatorFacadeFields {
                inner: self.fields.inner.clone(),
                registration: self.fields.registration.clone(),
                reference,
                virtual_allocations: self.fields.virtual_allocations.clone(),
            }),
        }
    }
    pub(super) fn source_observer(&self) -> Option<HandleObserver<dyn AllocatorSource>> {
        let observed = match &self.fields.reference {
            AllocatorReference::Owned(owner) => owner.observe(),
            AllocatorReference::Storage(_) => self.fields.registration.read().observe(),
            AllocatorReference::Policy => self.fields.registration.read().observe(),
        };
        (!observed.is_null()).then_some(observed)
    }
    pub(super) fn dispatch_source<R>(
        &self,
        call: impl FnOnce(&dyn AllocatorSource) -> R,
    ) -> Option<R> {
        let observed = self.source_observer()?;
        Some(call(&**observed))
    }
}
impl Clone for BaseAllocator {
    fn clone(&self) -> Self {
        let reference = match &self.fields.reference {
            AllocatorReference::Storage(lease) => AllocatorReference::Storage(lease.clone()),
            _ => self
                .source_observer()
                .map_or(AllocatorReference::Policy, |source| {
                    AllocatorReference::Owned(source.clone())
                }),
        };
        self.facade_with_reference(reference)
    }
}
// These façades never pin their independently allocated pointee. All shared policy mutation
// remains synchronized; unwind does not expose an intermediate typed allocation record.
impl Unpin for BaseAllocator {}
impl std::panic::UnwindSafe for BaseAllocator {}
impl std::panic::RefUnwindSafe for BaseAllocator {}
