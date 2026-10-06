use super::LocalArray;

impl<T: Copy, const N: usize> LocalArray<T, N> {
    /// Uses OCCT's trivial-element move assignment; `destination.move_from_trivial(&mut source)`.
    /// `Copy` selects this policy explicitly; it does not infer a C++ type's traits.
    pub fn move_from_trivial(&mut self, source: &mut Self) -> &mut Self {
        if !source.elements.spilled() {
            self.deallocate();
            self.elements.extend_from_slice(source.as_slice());
            source.elements.clear();
            return self;
        }
        if self.elements.spilled() {
            // OCCT swaps both heap pointers before zeroing the source's logical size.
            std::mem::swap(&mut self.elements, &mut source.elements);
            source.elements.clear();
            return self;
        }
        self.move_from(source);
        self
    }

    /// Preserves storage for native self move assignment; `array.move_self_trivial()`.
    pub fn move_self_trivial(&mut self) -> &mut Self {
        self.move_self()
    }
}
