//! Native Cell moves transfer object heads independently from LocalArray's storage policy.
use super::CellRecord;
use crate::ncollection::cell_filter::{CellGeometry, CellHash, CellIndex};
use crate::ncollection::{Array1, BaseAllocator};
impl<T> CellRecord<T> {
    pub(in crate::ncollection::cell_filter) fn from_point<I: CellGeometry<CellTarget = T>>(
        point: &I::CellPoint,
        sizes: &Array1<'_, f64>,
    ) -> Self {
        let mut cell = Self::from_coordinates(&[]);
        cell.index = CellIndex::from_point::<I>(point, sizes);
        cell
    }
    pub(in crate::ncollection::cell_filter) fn move_construct(source: &mut Self) -> Self {
        Self {
            index: CellIndex::move_construct(&mut source.index),
            objects: std::mem::replace(&mut source.objects, std::ptr::null()),
            ownership: std::marker::PhantomData,
        }
    }
    pub(in crate::ncollection::cell_filter) fn is_equal(&self, other: &Self) -> bool {
        self.index.as_slice() == other.index.as_slice()
    }
    pub(in crate::ncollection::cell_filter) fn hash_code(&self) -> usize {
        CellHash::hash_coordinates(self.index.as_slice())
    }
    pub(in crate::ncollection::cell_filter) fn move_from_discard(
        &mut self,
        source: &mut Self,
        allocator: &BaseAllocator,
    ) {
        self.index.move_from(&mut source.index);
        self.discard_objects(allocator);
        self.objects = std::mem::replace(&mut source.objects, std::ptr::null());
    }
    pub(in crate::ncollection::cell_filter) fn discard_objects(
        &mut self,
        allocator: &BaseAllocator,
    ) {
        let mut node = self.objects;
        self.objects = std::ptr::null();
        while !node.is_null() {
            let next = unsafe { (*node).next };
            // Native overwritten heads never invoke Target destruction. Rust only releases
            // typed storage leases here; an incremental arena retains its block bytes.
            unsafe { super::CellObjectNode::release_storage(node, allocator) };
            node = next;
        }
    }
    pub(in crate::ncollection::cell_filter) fn prepend(
        &mut self,
        target: &T,
        allocator: &BaseAllocator,
    ) where
        T: Clone,
    {
        unsafe { super::CellObjectNode::prepend(self, target, allocator) };
    }
    pub(in crate::ncollection::cell_filter) fn targets(&self) -> impl Iterator<Item = &T> {
        std::iter::successors((!self.objects.is_null()).then_some(self.objects), |node| {
            let next = unsafe { (**node).next };
            (!next.is_null()).then_some(next)
        })
        .map(|node| unsafe { &*std::ptr::addr_of!((*node).object).cast::<T>() })
    }
}
