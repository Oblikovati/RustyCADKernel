//! Native Cell moves transfer the target list independently from the index's storage policy.
use super::CellRecord;
use crate::ncollection::Array1;
use crate::ncollection::cell_filter::{CellGeometry, CellHash, CellIndex};
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
            objects: std::mem::take(&mut source.objects),
        }
    }
    pub(in crate::ncollection::cell_filter) fn is_equal(&self, other: &Self) -> bool {
        self.index.as_slice() == other.index.as_slice()
    }
    pub(in crate::ncollection::cell_filter) fn hash_code(&self) -> usize {
        CellHash::hash_coordinates(self.index.as_slice())
    }
    /// Native move assignment: the index moves and the old target list is overwritten without
    /// running the targets' destructors (a native leak of target-owned resources).
    pub(in crate::ncollection::cell_filter) fn move_from_discard(&mut self, source: &mut Self) {
        self.index.move_from(&mut source.index);
        self.discard_objects();
        self.objects = std::mem::take(&mut source.objects);
    }
    /// Forgets every target without its destructor, as a native overwritten list head does.
    pub(in crate::ncollection::cell_filter) fn discard_objects(&mut self) {
        for target in std::mem::take(&mut self.objects) {
            std::mem::forget(target);
        }
    }
    /// Native `add`: the copy becomes the new list head.
    pub(in crate::ncollection::cell_filter) fn prepend(&mut self, target: &T)
    where
        T: Clone,
    {
        self.objects.push(target.clone());
    }
    /// The targets in native list order, newest first.
    pub(in crate::ncollection::cell_filter) fn targets(&self) -> impl Iterator<Item = &T> {
        self.objects.iter().rev()
    }
}
