//! Appending publishes one element only after explicit relocation and construction succeed.
use super::LinearVector;

impl<T> LinearVector<T> {
    /// Appends with explicit native growth and value-construction policies;
    /// `vector.emplace_append_with_relocation(|v| *v, || 42)`.
    /// Select native move_if_noexcept via `relocate`, then copy, move, default or argument
    /// construction via `construct`. A final-constructor panic retains grown capacity and
    /// the old length. Growth panics retain `reserve_with_relocation`'s safe cleanup policy;
    /// native leaked unpublished replacement objects are not reproduced. Callbacks returning
    /// values do not promise address-dependent in-place construction.
    pub fn emplace_append_with_relocation(
        &mut self,
        relocate: impl FnMut(&mut T) -> T,
        construct: impl FnOnce() -> T,
    ) -> &mut T {
        self.reserve_with_relocation(self.len() + 1, relocate);
        self.push_reserved_item(construct())
    }
}
