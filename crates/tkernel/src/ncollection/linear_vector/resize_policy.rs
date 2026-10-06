//! Explicit resize operations retain native unpublished tail objects on constructor failure.
use super::LinearVector;

impl<T> LinearVector<T> {
    /// Resizes through native relocation and tail-construction policies;
    /// `vector.resize_with_relocation(4, |item| *item, || 0)`.
    /// Select move_if_noexcept through `relocate`, and default/fill construction through
    /// `construct`. Existing elements survive tail construction failures and grown capacity
    /// remains visible. Completed tail values remain live outside len on constructor panic,
    /// matching native Resize: later overwrite or storage release omits their destructors.
    /// This intentionally can leak resources. Shrink destructor panics retain Vec's safe cleanup,
    /// rather than native registered dead slots. Address-dependent construction is not modeled.
    pub fn resize_with_relocation(
        &mut self,
        size: usize,
        relocate: impl FnMut(&mut T) -> T,
        construct: impl FnMut() -> T,
    ) {
        self.reserve_with_relocation(size, relocate);
        self.resize_with_constructor(size, construct);
    }
}
