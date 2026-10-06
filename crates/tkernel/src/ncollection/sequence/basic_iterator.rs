//! Raw custom-node sequence cursors preserve native dispatch and detached pointer state.
use super::{BaseSequence, BaseSequenceRawIterator, BasicSequenceNode, SeqNode};
use std::marker::PhantomData;
mod access;
mod movement;

/// Native generic BasicIterator for arbitrary embedded sequence nodes.
/// Example: `BaseSequenceBasicIterator::<SequenceNode<i32>>::new(&base, true)`.
/// `MUTABLE=true` enables native writable dereference. Every item access remains unsafe:
/// copied cursors carry no ownership and cannot establish Rust reference exclusivity.
pub struct BaseSequenceBasicIterator<N: BasicSequenceNode, const MUTABLE: bool = false> {
    raw: BaseSequenceRawIterator,
    node_type: PhantomData<fn() -> N>,
}
impl<N: BasicSequenceNode, const M: bool> Copy for BaseSequenceBasicIterator<N, M> {}
impl<N: BasicSequenceNode, const M: bool> Clone for BaseSequenceBasicIterator<N, M> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<N: BasicSequenceNode, const M: bool> Default for BaseSequenceBasicIterator<N, M> {
    fn default() -> Self {
        Self::from_raw(BaseSequenceRawIterator::default())
    }
}
impl<N: BasicSequenceNode, const M: bool> BaseSequenceBasicIterator<N, M> {
    /// Initializes either endpoint without accessing a value; `Self::new(&base, false)`.
    pub fn new(base: &BaseSequence, is_start: bool) -> Self {
        Self::from_raw(BaseSequenceRawIterator::new(base, is_start))
    }
    /// Copies native Iterator links without dereferencing; `Self::from_raw(raw)`.
    pub fn from_raw(raw: BaseSequenceRawIterator) -> Self {
        Self {
            raw,
            node_type: PhantomData,
        }
    }
    /// Copies both links back to the base Iterator; `cursor.copy_to(&mut raw)`.
    pub fn copy_to(&self, raw: &mut BaseSequenceRawIterator) {
        *raw = self.raw;
    }
    /// Converts either specialization, including native historical const-to-mutable conversion.
    /// Example: `let writable = readonly.convert::<true>()`.
    pub fn convert<const OTHER: bool>(&self) -> BaseSequenceBasicIterator<N, OTHER> {
        BaseSequenceBasicIterator::from_raw(self.raw)
    }
    /// Assigns either native specialization; `cursor.assign(&other)`.
    pub fn assign<const OTHER: bool>(
        &mut self,
        other: &BaseSequenceBasicIterator<N, OTHER>,
    ) -> &mut Self {
        self.raw = other.raw;
        self
    }
    /// Returns the unchanged cursor for self assignment; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Resets to either endpoint; `cursor.init(&base, false)`.
    pub fn init(&mut self, base: &BaseSequence, is_start: bool) {
        self.raw.init(base, is_start);
    }
    /// Uses the native Initialize spelling; `cursor.initialize(&base, true)`.
    pub fn initialize(&mut self, base: &BaseSequence, is_start: bool) {
        self.init(base, is_start);
    }
    /// Reports native More without traversing a link; `cursor.more()`.
    pub fn more(&self) -> bool {
        !self.raw.current_node().is_null()
    }
    /// Returns the embedded current base link, with no concrete-node cast; `cursor.current_node()`.
    pub fn current_node(&self) -> *const SeqNode {
        self.raw.current_node().cast_const()
    }
    /// Returns the remembered previous base link; `cursor.previous_node()`.
    pub fn previous_node(&self) -> *const SeqNode {
        self.raw.previous_node().cast_const()
    }
    /// Compares only current links across const specializations; `cursor.is_equal(&other)`.
    pub fn is_equal<const OTHER: bool>(&self, other: &BaseSequenceBasicIterator<N, OTHER>) -> bool {
        self.current_node() == other.current_node()
    }
}
impl<N: BasicSequenceNode, const M: bool, const OTHER: bool>
    PartialEq<BaseSequenceBasicIterator<N, OTHER>> for BaseSequenceBasicIterator<N, M>
{
    fn eq(&self, other: &BaseSequenceBasicIterator<N, OTHER>) -> bool {
        self.is_equal(other)
    }
}
impl<N: BasicSequenceNode, const M: bool> Eq for BaseSequenceBasicIterator<N, M> {}
