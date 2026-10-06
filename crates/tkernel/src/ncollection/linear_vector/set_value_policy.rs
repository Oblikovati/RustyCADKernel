//! SetValue publishes default extension before invoking native copy/move assignment.
use super::LinearVector;

impl<T> LinearVector<T> {
    /// Assigns an existing slot or extends through explicit native element policies;
    /// `vector.set_value_with_policies(2, |v| *v, || 0, |v| *v = 7)`.
    /// Successful extension publishes its full length before assignment. If assignment
    /// panics, its valid partial mutations and the extended length remain observable.
    /// Failed extension retains `resize_with_relocation`'s temporary safe rollback policy;
    /// native unpublished-tail preservation remains tracked in the lifetime follow-up plan.
    pub fn set_value_with_policies(
        &mut self,
        index: usize,
        relocate: impl FnMut(&mut T) -> T,
        construct_default: impl FnMut() -> T,
        assign: impl FnOnce(&mut T),
    ) -> &mut T {
        if index >= self.len() {
            let size = index.checked_add(1).unwrap_or_else(|| {
                panic!("SetValue index {index}: expected index below usize::MAX")
            });
            self.resize_with_relocation(size, relocate, construct_default);
        }
        let target = &mut self.elements[index];
        assign(target);
        target
    }
}
