//! Native Object destruction precedes unlink; callback failures preserve the current node.
use super::{BaseAllocator, BaseMap, CellIndex, CellMap, CellObjectNode, cell_node::CellRecord};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
impl<T> CellMap<T> {
    pub(in crate::ncollection::cell_filter) fn try_retain<E>(
        &mut self,
        index: &CellIndex,
        keep: impl FnMut(&T) -> Result<bool, E>,
    ) -> Result<(), E> {
        self.try_retain_index(index.as_slice(), keep)
    }
    pub(in crate::ncollection::cell_filter) fn try_retain_index<E>(
        &mut self,
        index: &[i32],
        keep: impl FnMut(&T) -> Result<bool, E>,
    ) -> Result<(), E> {
        let Some((bucket, node)) = self.lookup_index(index) else {
            return Ok(());
        };
        let cell = unsafe { super::CellMapNode::cell_ptr(node.cast()) };
        let result = catch_unwind(AssertUnwindSafe(|| unsafe {
            retain_objects(cell, self.storage.base.allocator(), keep)
        }));
        self.finish_retention(cell, bucket, node, result)
    }
    fn finish_retention<E>(
        &mut self,
        cell: *mut CellRecord<T>,
        bucket: usize,
        node: *mut super::MapLinkNode,
        result: std::thread::Result<Result<(), E>>,
    ) -> Result<(), E> {
        // An already-destroyed final Object must not leave a dead cell after Rust unwinding.
        if unsafe { (*cell).objects.is_null() } {
            self.storage.remove(bucket, node);
        }
        match result {
            Ok(outcome) => outcome,
            Err(panic) => resume_unwind(panic),
        }
    }
}
unsafe fn retain_objects<T, E>(
    cell: *mut CellRecord<T>,
    allocator: &BaseAllocator,
    mut keep: impl FnMut(&T) -> Result<bool, E>,
) -> Result<(), E> {
    let (mut previous, mut node) = (std::ptr::null(), unsafe { (*cell).objects });
    while !node.is_null() {
        let next = unsafe { (*node).next };
        if keep(unsafe { &(*node).object })? {
            previous = node;
        } else {
            unsafe { purge_object(cell, previous, node, next, allocator) };
        }
        node = next;
    }
    Ok(())
}
unsafe fn purge_object<T>(
    cell: *mut CellRecord<T>,
    previous: *const CellObjectNode<T>,
    node: *const CellObjectNode<T>,
    next: *const CellObjectNode<T>,
    allocator: &BaseAllocator,
) {
    let dropped = catch_unwind(AssertUnwindSafe(|| unsafe {
        CellObjectNode::drop_object(node)
    }));
    unsafe { publish_unlink(cell, previous, next) };
    unsafe { CellObjectNode::release_storage(node, allocator) };
    if let Err(panic) = dropped {
        resume_unwind(panic);
    }
}

unsafe fn publish_unlink<T>(
    cell: *mut CellRecord<T>,
    previous: *const CellObjectNode<T>,
    next: *const CellObjectNode<T>,
) {
    if previous.is_null() {
        unsafe { (*cell).objects = next };
    } else {
        unsafe { (*previous.cast_mut()).next = next };
    }
}
