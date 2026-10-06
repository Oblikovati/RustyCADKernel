//! Explicit resize element operations reuse the established safe rollback policy.
use super::LinearVector;

impl<T> LinearVector<T> {
    /// Resizes through native relocation and tail-construction policies;
    /// `vector.resize_with_relocation(4, |item| *item, || 0)`.
    /// Select move_if_noexcept through `relocate`, and default/fill construction through
    /// `construct`. Existing elements survive tail construction failures and grown capacity
    /// remains visible. Rust drops completed tail values on failure; native Resize instead
    /// leaves them live outside Size. Shrink destructor panics retain Vec's safe cleanup,
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
