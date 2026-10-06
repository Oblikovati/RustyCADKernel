//! Fallible native callbacks retain earlier purges and stop at the throwing target.
use super::{CellAction, CellFilter, CellGeometry, CellIndex};
impl<I: CellGeometry> CellFilter<I> {
    /// Visit one cell until callback failure; `filter.try_inspect_with(&point, |target| Ok::<_, ()>(CellAction::Keep))?`.
    pub fn try_inspect_with<E>(
        &mut self,
        point: &I::CellPoint,
        inspect: impl FnMut(&I::CellTarget) -> Result<CellAction, E>,
    ) -> Result<(), E> {
        self.try_inspect_with_cell(&self.indices(point), inspect)
    }
    /// Visit a range in native axis order, stopping at failure; `filter.try_inspect_range_with(&min,&max, |target| Ok::<_, ()>(CellAction::Purge))?`.
    pub fn try_inspect_range_with<E>(
        &mut self,
        min: &I::CellPoint,
        max: &I::CellPoint,
        mut inspect: impl FnMut(&I::CellTarget) -> Result<CellAction, E>,
    ) -> Result<(), E> {
        for index in self.range(min, max) {
            self.try_inspect_with_cell(&index, &mut inspect)?;
        }
        Ok(())
    }
    fn try_inspect_with_cell<E>(
        &mut self,
        index: &CellIndex,
        mut inspect: impl FnMut(&I::CellTarget) -> Result<CellAction, E>,
    ) -> Result<(), E> {
        self.try_retain_cell(index, |stored| {
            inspect(stored).map(|action| action == CellAction::Keep)
        })
    }
    pub(super) fn try_retain_cell<E>(
        &mut self,
        index: &CellIndex,
        keep: impl FnMut(&I::CellTarget) -> Result<bool, E>,
    ) -> Result<(), E> {
        self.cells.try_retain(index, keep)
    }
}
