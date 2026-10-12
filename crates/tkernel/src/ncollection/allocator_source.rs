//! Genuine NCollection_BaseAllocator virtual dispatch through its embedded native base.
use super::{AllocatorObject, AllocatorStorageLease};
use crate::standard::{RuntimeType, RuntimeTyped, Transient, project_interface};
/// Native allocator contract; implement on a real descendant and register its interface caster.
/// Example: `BaseAllocator::from_handle(&Handle::new(MyAllocator::default())).allocate_raw(32)`.
///
/// # Safety
/// Returned ranges must be disjoint, live and large enough for the requested byte count until
/// matching Free; implementation must uphold its documented reset and thread-safety policy.
/// `physical_storage` must retain and release those ranges independently of this object's lifetime;
/// custom providers must override the default lease when allocating outside the embedded policy.
/// `dispatch_code` and `physical_storage` are pure internal metadata accessors, without callbacks.
/// Virtual bodies execute through AllocatorInvocation; actual field-access scopes must end before
/// callbacks can destroy the logical receiver. They must not snapshot later-reread mutable slots.
pub unsafe trait AllocatorSource: Transient {
    /// Borrow the genuine parent; `self.allocator_object_base().allocate_base(32)`.
    fn allocator_object_base(&self) -> &AllocatorObject;
    /// Lease physical allocation memory independently; `let memory=source.physical_storage()`.
    fn physical_storage(&self) -> AllocatorStorageLease {
        let parent = self.allocator_object_base();
        parent.resource().physical_storage()
    }
    /// Copy immutable virtual code; `source.dispatch_code()`.
    /// This internal accessor must not invoke callbacks or release native resources.
    fn dispatch_code(&self) -> super::AllocatorDispatchCode {
        super::AllocatorDispatchCode::default()
    }
}
impl RuntimeTyped for dyn AllocatorSource {
    const TYPE_NAME: &'static str = "NCollection_BaseAllocator";
    fn static_type() -> &'static RuntimeType {
        // The owned interface aliases the real concrete base descriptor, not a second native class.
        AllocatorObject::static_type()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        let transient = object;
        project_interface::<Self>(transient)
    }
}
