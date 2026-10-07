//! Native assignment clears the receiver first and commits each successfully constructed node.
use super::Sequence;

impl<T: Clone> Sequence<T> {
    /// Copies elements into this allocator; `sequence.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        self.assign_with_constructor(source, T::clone)
    }
}
impl<T> Sequence<T> {
    /// Copies with an explicit item constructor and keeps this allocator; `target.assign_with_constructor(&source, |x| *x)`.
    /// Old items are destroyed before the first copy. On callback panic the receiver owns
    /// its completed prefix; only the currently uninitialized allocation is freed. The
    /// distinct-container borrow domain excludes self assignment; use `assign_self()` for it.
    pub fn assign_with_constructor(
        &mut self,
        source: &Self,
        mut construct: impl FnMut(&T) -> T,
    ) -> &mut Self {
        self.clear(None);
        for value in source.iter() {
            self.emplace_append(|| construct(value));
        }
        self
    }
}

#[cfg(test)]
mod tests;
