//! Map<Cell> destruction receives its allocator from the canonical node owner.
use super::{CellIndex, CellObjectNode, MapNode};
use crate::ncollection::destructor_unwind::DeferredDestructorPanic;
use crate::ncollection::owned_bucket_chains::{CanonicalMapNode, node_contract::sealed};
use crate::ncollection::{BaseAllocator, BasicMapNode};
#[repr(C)]
pub(in crate::ncollection::cell_filter) struct CellRecord<T> {
    pub(in crate::ncollection::cell_filter) index: CellIndex,
    pub(super) objects: *const CellObjectNode<T>,
    ownership: std::marker::PhantomData<Box<T>>,
}
impl<T> CellRecord<T> {
    pub(in crate::ncollection::cell_filter) fn from_coordinates(index: &[i32]) -> Self {
        Self {
            index: CellIndex::from_slice(index),
            objects: std::ptr::null(),
            ownership: std::marker::PhantomData,
        }
    }
    #[cfg(test)]
    pub(in crate::ncollection::cell_filter) fn len(&self) -> usize {
        let (mut count, mut node) = (0, self.objects);
        while !node.is_null() {
            count += 1;
            node = unsafe { (*node).next };
        }
        count
    }
}
// SAFETY: this private record owns its reachable targets; mutation requires the exclusive
// CellMap or standalone CellFilterCell owner.
// No public map-key reference or arbitrary head mutation escapes this sealed representation.
unsafe impl<T: Send> Send for CellRecord<T> {}
unsafe impl<T: Sync> Sync for CellRecord<T> {}
#[repr(transparent)]
pub(super) struct CellMapNode<T>(pub(super) MapNode<CellRecord<T>>);
impl<T> BasicMapNode for CellMapNode<T> {
    type Key = CellRecord<T>;
    type Value = CellRecord<T>;
    fn bucket_next(&self) -> *mut Self {
        self.0.link().next().cast()
    }
    fn order_next(&self) -> *mut Self {
        std::ptr::null_mut()
    }
    fn map_key(&self) -> &Self::Key {
        self.0.key()
    }
    fn map_value(&self) -> &Self::Value {
        self.0.key()
    }
}
impl<T> sealed::Sealed for CellMapNode<T> {}
// SAFETY: transparent MapNode preserves its real link prefix and exact allocated concrete type.
unsafe impl<T> CanonicalMapNode for CellMapNode<T> {
    unsafe fn release_owned(pointer: *mut Self, allocator: &BaseAllocator) {
        let cell = unsafe { Self::cell_ptr(pointer) };
        let mut panics = DeferredDestructorPanic::default();
        unsafe { CellRecord::drop_objects(cell, allocator, &mut panics) };
        panics.capture(|| unsafe {
            crate::ncollection::node_release::release_typed_node(pointer, allocator)
        });
        panics.finish(Ok(()));
    }
}
impl<T> CellMapNode<T> {
    pub(super) unsafe fn cell_ptr(node: *mut Self) -> *mut CellRecord<T> {
        unsafe { MapNode::key_ptr(node.cast()) }
    }
}
mod record_methods;
impl<T> CellRecord<T> {
    pub(in crate::ncollection::cell_filter) unsafe fn drop_objects(
        cell: *mut CellRecord<T>,
        allocator: &BaseAllocator,
        panics: &mut DeferredDestructorPanic,
    ) {
        let mut node = unsafe { (*cell).objects };
        while !node.is_null() {
            let next = unsafe { (*node).next };
            panics.capture(|| unsafe { CellObjectNode::drop_object(node) });
            unsafe { CellObjectNode::release_storage(node, allocator) };
            node = next;
        }
        // Native Cell::~Cell preserves its original head through every target destructor.
        unsafe { (*cell).objects = std::ptr::null() };
    }
}
