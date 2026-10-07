//! Explicit queued construction preserves the native temporary lifetime and allocation phase.
use super::{BoundingTreeInsertion, BoundingVolume, ObjBnd, UBTreeFiller};
impl<T: Clone + Default, B: BoundingVolume, Tree: BoundingTreeInsertion<T, B> + ?Sized>
    UBTreeFiller<'_, T, B, Tree>
{
    /// Copies inputs into the native temporary, then constructs the queued value from it.
    /// Choose the payload's native copy/move behavior explicitly; the temporary drops afterward.
    /// Example: `filler.add_ref_with_constructor(&object, &bounds, |pair| pair.clone())`.
    pub fn add_ref_with_constructor(
        &mut self,
        object: &T,
        bounds: &B,
        construct: impl FnOnce(&mut ObjBnd<T, B>) -> ObjBnd<T, B>,
    ) {
        let mut temporary = ObjBnd::new_ref(object, bounds);
        self.pending
            .append_moved_with_constructor(&mut temporary, construct);
    }
}
