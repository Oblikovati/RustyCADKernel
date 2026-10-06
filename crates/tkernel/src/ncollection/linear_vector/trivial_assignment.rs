use super::LinearVector;

impl<T: Copy> LinearVector<T> {
    /// Uses native trivial-element byte copying without Clone hooks; `destination.assign_trivial(&source)`.
    /// The caller explicitly selects the C++ trivially-copyable policy with Rust Copy.
    pub fn assign_trivial(&mut self, source: &Self) -> &mut Self {
        if source.len() > self.capacity() {
            // OCCT copy construction reserves exactly the source length, not its capacity.
            *self = source.clone_trivial();
            return self;
        }
        self.elements.clear();
        // Copied reads Copy values directly; Vec::extend_from_slice can dispatch Clone.
        self.elements.extend(source.as_slice().iter().copied());
        self
    }

    /// Copies initialized values into exactly sized storage without Clone hooks; `vector.clone_trivial()`.
    pub fn clone_trivial(&self) -> Self {
        let mut elements = Vec::with_capacity(self.len());
        elements.extend(self.as_slice().iter().copied());
        Self { elements }
    }
}
