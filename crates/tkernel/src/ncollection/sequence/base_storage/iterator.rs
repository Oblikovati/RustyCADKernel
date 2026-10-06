use super::{BaseSequence, SeqNode};

/// Raw BaseSequence::Iterator links; `BaseSequenceRawIterator::new(&base, false)`.
#[derive(Clone, Copy, Debug)]
pub struct BaseSequenceRawIterator {
    /// Native protected current link for derived cursor composition.
    pub current: *mut SeqNode,
    /// Native protected previous link for derived cursor composition.
    pub previous: *mut SeqNode,
}
impl Default for BaseSequenceRawIterator {
    fn default() -> Self {
        Self {
            current: std::ptr::null_mut(),
            previous: std::ptr::null_mut(),
        }
    }
}
impl BaseSequenceRawIterator {
    /// Initializes the first or past-last state; `Self::new(&base, true)`.
    pub fn new(base: &BaseSequence, is_start: bool) -> Self {
        if is_start {
            return Self {
                current: base.first,
                previous: std::ptr::null_mut(),
            };
        }
        Self {
            current: std::ptr::null_mut(),
            previous: base.last,
        }
    }
    /// Resets both links; `iterator.init(&base, false)`.
    pub fn init(&mut self, base: &BaseSequence, is_start: bool) {
        *self = Self::new(base, is_start);
    }
    /// Returns CurrentNode without dereferencing it; `iterator.current_node()`.
    pub fn current_node(&self) -> *mut SeqNode {
        self.current
    }
    /// Returns PreviousNode without dereferencing it; `iterator.previous_node()`.
    pub fn previous_node(&self) -> *mut SeqNode {
        self.previous
    }
    /// Implements native Previous, including the end-to-tail step; `unsafe { iterator.previous() }`.
    /// # Safety
    /// The remembered previous node must be null or live with readable native links.
    pub unsafe fn previous(&mut self) {
        self.current = self.previous;
        if !self.current.is_null() {
            self.previous = unsafe { (*self.current).previous() };
        }
    }
    /// Advances using the typed BasicIterator operation; `unsafe { iterator.next() }`.
    /// # Safety
    /// Current must be null or live with readable native links.
    pub unsafe fn next(&mut self) {
        if self.current.is_null() {
            return;
        }
        self.previous = self.current;
        self.current = unsafe { (*self.current).next() };
    }
}
