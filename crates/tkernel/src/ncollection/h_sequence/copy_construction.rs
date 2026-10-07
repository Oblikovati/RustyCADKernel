use super::{HSequence, RwLock, Sequence, StandardTransient};

impl<T: Clone> HSequence<T> {
    /// Copies `source` and keeps its allocator, as the C++ copy of the base sequence does;
    /// `HSequence::from_sequence(&values)`.
    pub fn from_sequence(source: &Sequence<T>) -> Self {
        Self {
            base: StandardTransient::default(),
            items: RwLock::new(source.clone()),
        }
    }
}
impl<T> HSequence<T> {
    /// Uses the native base copy-constructor lifecycle with an explicit item policy.
    /// `HSequence::from_sequence_with_constructor(&values, |value| *value)`.
    /// Callback panic abandons completed copies without payload destruction, as the native
    /// failed base constructor does. No wrapper or safe node reference escapes that failure.
    pub fn from_sequence_with_constructor(
        source: &Sequence<T>,
        construct: impl FnMut(&T) -> T,
    ) -> Self {
        Self::from_owned(Sequence::from_copied_with(source, construct))
    }
}
