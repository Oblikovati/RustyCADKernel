//! Returning iterator operations preserve both OCCT cursor links.
use super::{Sequence, SequenceCursor};

impl SequenceCursor {
    /// Exposes the opaque native CurrentNode identity; `cursor.current_node()`.
    pub fn current_node(&self) -> Option<usize> {
        self.address
    }

    /// Exposes the opaque native PreviousNode identity; `cursor.previous_node()`.
    pub fn previous_node(&self) -> Option<usize> {
        self.previous
    }

    /// Initializes either endpoint through OCCT's Initialize spelling; `cursor.initialize(&sequence, false)`.
    pub fn initialize<T>(&mut self, sequence: &Sequence<T>, is_start: bool) {
        self.init(sequence, is_start);
    }

    /// Copies both current and previous links; `cursor.copy_to(&mut other)`.
    pub fn copy_to(self, destination: &mut Self) {
        *destination = self;
    }

    /// Assigns both cursor links and returns the destination; `cursor.assign(&other)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        *self = *source;
        self
    }

    /// Preserves cursor links for self assignment; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }

    /// Tests node identity, including equal end cursors from different sequences; `cursor.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self == other
    }

    /// Advances and returns this cursor, matching prefix increment; `cursor.advance(&sequence)`.
    pub fn advance<T>(&mut self, sequence: &Sequence<T>) -> &mut Self {
        self.next(sequence);
        self
    }

    /// Advances and returns the previous cursor, matching postfix increment; `let old = cursor.post_advance(&sequence)`.
    pub fn post_advance<T>(&mut self, sequence: &Sequence<T>) -> Self {
        let previous = *self;
        self.next(sequence);
        previous
    }

    /// Retreats and returns this cursor, matching prefix decrement; `cursor.retreat(&sequence)`.
    pub fn retreat<T>(&mut self, sequence: &Sequence<T>) -> &mut Self {
        self.previous(sequence);
        self
    }

    /// Retreats and returns the previous cursor, matching postfix decrement; `let old = cursor.post_retreat(&sequence)`.
    pub fn post_retreat<T>(&mut self, sequence: &Sequence<T>) -> Self {
        let previous = *self;
        self.previous(sequence);
        previous
    }
}
