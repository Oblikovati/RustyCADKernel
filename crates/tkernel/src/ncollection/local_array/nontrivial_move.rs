//! Explicit callbacks preserve C++ nontrivial move-constructor observations.
use super::LocalArray;
use smallvec::SmallVec;

impl<T: Default, const N: usize> LocalArray<T, N> {
    /// Reallocates with native nontrivial move callbacks; e.g.
    /// `array.reallocate_with_move(8, true, |item| std::mem::take(item))`.
    /// The callback must leave its source valid for destruction. Non-panicking
    /// callbacks and defaults match OCCT's successful construction domain.
    pub fn reallocate_with_move<F>(&mut self, size: usize, copy: bool, mut construct: F)
    where
        F: FnMut(&mut T) -> T,
    {
        if size <= self.len() || (!self.elements.spilled() && size <= N) {
            self.reallocate(size, copy);
            return;
        }
        let mut replacement = SmallVec::with_capacity(size);
        if copy {
            replacement.extend(self.elements.iter_mut().map(&mut construct));
        }
        replacement.resize_with(size, T::default);
        self.truncate_forward(0);
        self.elements = replacement;
    }
}

impl<T, const N: usize> LocalArray<T, N> {
    /// Constructs from a source using native inline move callbacks; e.g.
    /// `LocalArray::move_construct_with(&mut source, |item| std::mem::take(item))`.
    /// Heap storage transfers directly. Inline sources remain valid until all
    /// callbacks complete, then their moved-from values are destroyed in order.
    pub fn move_construct_with<F>(source: &mut Self, mut construct: F) -> Self
    where
        F: FnMut(&mut T) -> T,
    {
        if source.elements.spilled() {
            return std::mem::take(source);
        }
        let mut destination = Self::default();
        destination
            .elements
            .extend(source.elements.iter_mut().map(&mut construct));
        source.truncate_forward(0);
        destination
    }

    /// Destroys the receiver before moving from the source; e.g.
    /// `destination.move_from_with(&mut source, |item| std::mem::take(item))`.
    /// The callback models a native nonthrowing move constructor and must leave
    /// its source valid for destruction. Self assignment uses `move_self()`.
    pub fn move_from_with<F>(&mut self, source: &mut Self, construct: F) -> &mut Self
    where
        F: FnMut(&mut T) -> T,
    {
        self.deallocate();
        *self = Self::move_construct_with(source, construct);
        self
    }
}
