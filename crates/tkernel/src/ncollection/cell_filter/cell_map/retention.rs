//! Native `remove`/`inspect`: walk one cell's targets newest first, destroy each purged target
//! as it is met, and drop the cell from the map once its list is empty.
use super::{CellIndex, CellMap};
impl<T> CellMap<T> {
    pub(in crate::ncollection::cell_filter) fn try_retain<E>(
        &mut self,
        index: &CellIndex,
        keep: impl FnMut(&T) -> Result<bool, E>,
    ) -> Result<(), E> {
        self.try_retain_index(index.as_slice(), keep)
    }
    /// Rust-only fallible form: a `keep` error stops the walk; purges already made stay made.
    pub(in crate::ncollection::cell_filter) fn try_retain_index<E>(
        &mut self,
        index: &[i32],
        keep: impl FnMut(&T) -> Result<bool, E>,
    ) -> Result<(), E> {
        let Some(slot) = self.lookup_index(index) else {
            return Ok(());
        };
        let cell = self.storage.payload_mut().key_mut(slot);
        let outcome = retain_newest_first(&mut cell.objects, keep);
        if cell.is_vacant() {
            self.remove_slot(slot);
        }
        outcome
    }
}
// The vector holds the native list reversed, so its last element is the list head.
fn retain_newest_first<T, E>(
    objects: &mut Vec<T>,
    mut keep: impl FnMut(&T) -> Result<bool, E>,
) -> Result<(), E> {
    let mut position = objects.len();
    while position > 0 {
        position -= 1;
        if !keep(&objects[position])? {
            // Native destroys the target, then unlinks its node; the vector is consistent
            // before the destructor runs.
            drop(objects.remove(position));
        }
    }
    Ok(())
}
