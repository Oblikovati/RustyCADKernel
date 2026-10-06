//! Native target nodes store Object first and Next last; no allocator handle is embedded.
use super::cell_node::CellRecord;
use crate::ncollection::{BaseAllocator, node_owner::NativeNodeOwner};
use std::mem::ManuallyDrop;
#[repr(C)]
pub(super) struct CellObjectNode<T> {
    pub(super) object: ManuallyDrop<T>,
    // Readonly link type preserves target covariance; only the unique CellMap owner mutates it.
    pub(super) next: *const Self,
}
impl<T: Clone> CellObjectNode<T> {
    pub(super) unsafe fn prepend(cell: *mut CellRecord<T>, target: &T, allocator: &BaseAllocator) {
        let owner = allocator.allocate_value_with(|| Self {
            object: ManuallyDrop::new(target.clone()),
            // Native reads the previous head only after target copy construction completes.
            next: unsafe { (*cell).objects },
        });
        let (node, _resource) = NativeNodeOwner::from_allocated(owner).into_parts();
        unsafe { (*cell).objects = node.cast_const() };
    }
}
impl<T> CellObjectNode<T> {
    pub(super) unsafe fn drop_object(node: *const Self) {
        // Project only Object; cached Next and the canonical head remain readable during Drop.
        unsafe { ManuallyDrop::drop(&mut (*node.cast_mut()).object) };
    }
    pub(super) unsafe fn release_storage(node: *const Self, allocator: &BaseAllocator) {
        // ManuallyDrop suppresses a second target Drop. Incremental deallocation only releases
        // Rust reset protection; it neither reclaims block bytes nor calls native Free.
        unsafe { crate::ncollection::node_release::release_typed_node(node.cast_mut(), allocator) };
    }
}
