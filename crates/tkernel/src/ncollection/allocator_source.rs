//! Genuine NCollection_BaseAllocator virtual dispatch through its embedded native base.
use super::{AllocatorObject, AllocatorStorageLease};
use crate::standard::{Failure, RuntimeType, RuntimeTyped, Transient, project_interface};
use std::ptr::NonNull;
/// Native allocator contract; implement on a real descendant and register its interface caster.
/// Example: `BaseAllocator::from_handle(&Handle::new(MyAllocator::default())).allocate_raw(32)`.
///
/// # Safety
/// Returned ranges must be disjoint, live and large enough for the requested byte count until
/// matching Free; implementation must uphold its documented reset and thread-safety policy.
/// `physical_storage` must retain and release those ranges independently of this object's lifetime;
/// custom providers must override the default lease when allocating outside the embedded policy.
pub unsafe trait AllocatorSource: Transient {
    /// Borrow the genuine parent; `self.allocator_object_base().allocate_base(32)`.
    fn allocator_object_base(&self) -> &AllocatorObject;
    /// Lease physical allocation memory independently; `let memory=source.physical_storage()`.
    fn physical_storage(&self) -> AllocatorStorageLease {
        let parent = self.allocator_object_base();
        parent.resource().physical_storage()
    }
    /// Dispatch native Allocate; `source.allocate(32)`.
    fn allocate(&self, size: usize) -> Option<NonNull<u8>> {
        let parent = self.allocator_object_base();
        if parent.allocate_calls_optimal() {
            return self.allocate_optimal(size);
        }
        parent.allocate_base(size)
    }
    /// Dispatch native AllocateOptimal independently of Allocate; `source.allocate_optimal(32)`.
    fn allocate_optimal(&self, size: usize) -> Option<NonNull<u8>> {
        let parent = self.allocator_object_base();
        if parent.optimal_calls_allocate() {
            return self.allocate(size);
        }
        parent.allocate_optimal_base(size)
    }
    /// Dispatch nullable native Free; `unsafe { source.free(pointer) }?`.
    /// # Safety
    /// Non-null pointers must be a live allocation accepted by this actual allocator instance.
    unsafe fn free(&self, pointer: *mut u8) -> Result<(), Failure> {
        let parent = self.allocator_object_base();
        // SAFETY: the caller supplies a valid nullable range for this allocator's base policy.
        unsafe { parent.free_base(pointer) }
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
