//! Borrowed single-item construction shares the existing allocator-before-constructor node path.
use super::HSequence;

impl<T> HSequence<T> {
    /// Allocates a node before explicitly copy-constructing a borrowed donor.
    /// Callback panic releases the new node, retaining the existing sequence and caller-owned donor.
    /// `items.append_ref_with_constructor(&donor, |source| Item::new(source.number))`.
    pub fn append_ref_with_constructor(&self, donor: &T, construct: impl FnOnce(&T) -> T) {
        self.change_sequence(|items| {
            items.emplace_append(|| construct(donor));
        });
    }
}

impl<T: Clone> HSequence<T> {
    /// Copies a borrowed item after allocating its node, native Append(const T&); `items.append_ref(&42)`.
    pub fn append_ref(&self, value: &T) {
        self.append_ref_with_constructor(value, T::clone);
    }
}
