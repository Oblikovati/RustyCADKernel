//! Owning OCCT 3d097a UniversalIterator adaptation with nullable polymorphic clone hooks.
use super::ParallelIteratorInterface;

/// Owns an optional polymorphic iterator; `let empty = ParallelUniversalIterator::new();`.
/// Empty dereference is valid; cloning, comparison and increment require a stored iterator.
/// Safe access separates shared and mutable borrows, unlike native pointer dereference.
pub struct ParallelUniversalIterator<'data> {
    iterator: Option<Box<dyn ParallelIteratorInterface<'data> + 'data>>,
}

impl<'data> ParallelUniversalIterator<'data> {
    /// Creates the native default empty state; `assert!(ParallelUniversalIterator::new().value().is_none())`.
    pub fn new() -> Self {
        Self { iterator: None }
    }

    /// Takes wrapper ownership, including null; `let empty = ParallelUniversalIterator::from_interface(None);`.
    pub fn from_interface(
        iterator: Option<Box<dyn ParallelIteratorInterface<'data> + 'data>>,
    ) -> Self {
        Self { iterator }
    }

    /// Borrows the stored interface or null state; `let interface = iterator.value();`.
    pub fn value(&self) -> Option<&(dyn ParallelIteratorInterface<'data> + 'data)> {
        self.iterator.as_deref()
    }

    /// Exclusively borrows the stored interface; `iterator.value_mut().unwrap().increment();`.
    pub fn value_mut(&mut self) -> Option<&mut (dyn ParallelIteratorInterface<'data> + 'data)> {
        self.iterator.as_deref_mut()
    }

    /// Clones before replacing the old wrapper; `target.assign(&source).increment();`.
    /// A clone panic leaves the original target intact, as a native throwing Clone does.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        let copy = source.required_value("assign").clone_box();
        self.iterator = copy;
        self
    }

    /// Models native self-assignment without aliased mutable borrows; `iterator.assign_self();`.
    pub fn assign_self(&mut self) -> &mut Self {
        let copy = self.required_value("assign_self").clone_box();
        self.iterator = copy;
        self
    }

    /// Advances and returns this same wrapper; `iterator.increment().increment();`.
    pub fn increment(&mut self) -> &mut Self {
        self.value_mut()
            .expect("UniversalIterator increment: observed empty interface; expected initialized interface")
            .increment();
        self
    }

    /// Clones the old state before advancing; `let previous = iterator.post_increment();`.
    pub fn post_increment(&mut self) -> Self {
        let previous = self.clone();
        self.increment();
        previous
    }

    fn required_value(&self, operation: &str) -> &(dyn ParallelIteratorInterface<'data> + 'data) {
        self.value().unwrap_or_else(|| {
            panic!("UniversalIterator {operation}: observed empty interface; expected initialized interface")
        })
    }
}

impl Default for ParallelUniversalIterator<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ParallelUniversalIterator<'_> {
    fn clone(&self) -> Self {
        Self::from_interface(self.required_value("clone").clone_box())
    }

    fn clone_from(&mut self, source: &Self) {
        self.assign(source);
    }
}

impl PartialEq for ParallelUniversalIterator<'_> {
    // OCCT has a separate operator!= declaration; retain its exact executable binding.
    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }

    fn eq(&self, other: &Self) -> bool {
        self.required_value("equality")
            .is_equal(other.required_value("equality rhs"))
    }
}
