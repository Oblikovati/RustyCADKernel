//! Explicit self-argument operations represent OCCT aliasing branches without aliased borrows.
use super::{Sequence, out_of_range};
use crate::standard::Failure;

impl<T> Sequence<T> {
    /// Returns the native unsigned Size count; `assert_eq!(sequence.size(), sequence.len())`.
    pub fn size(&self) -> usize {
        self.len()
    }

    /// Creates the base bidirectional iterator specialization; `sequence.base_iterator::<false>(true)`.
    pub fn base_iterator<const MUTABLE: bool>(
        &self,
        is_start: bool,
    ) -> super::BaseSequenceIterator<T, MUTABLE> {
        super::BaseSequenceIterator::new(self, is_start)
    }

    /// Preserves nodes for self copy/move assignment; `sequence.assign_self().append(value)`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }

    /// Preserves a sequence appended or prepended to itself; `sequence.append_self()`.
    pub fn append_self(&mut self) {}

    /// Preserves a sequence prepended to itself; `sequence.prepend_self()`.
    pub fn prepend_self(&mut self) {}

    /// Preserves self insertion before any index validation, as OCCT does; `sequence.insert_self_after(usize::MAX)`.
    pub fn insert_self_after(&mut self, _index: usize) {}

    /// Rejects zero before taking OCCT's self-insertion branch; `sequence.insert_self_before(1)?`.
    pub fn insert_self_before(&mut self, index: usize) -> Result<(), Failure> {
        if index == 0 {
            return Err(out_of_range("InsertBefore", index, "an index >= 1"));
        }
        Ok(())
    }

    /// Preserves OCCT's clearing before a failed self split; `sequence.split_self(1)` clears then errors.
    pub fn split_self(&mut self, index: usize) -> Result<(), Failure> {
        self.clear(None);
        self.offset("Split", index)?;
        Ok(())
    }
}
