//! Signed OCCT overloads reject negative indices before delegating to size_t behavior.
use super::Sequence;
use crate::standard::Failure;

fn unsigned_sequence_index(member: &str, index: i32) -> Result<usize, Failure> {
    usize::try_from(index).map_err(|_| {
        Failure::out_of_range(&format!(
            "NCollection_Sequence::{member} index {index}; expected a nonnegative index"
        ))
    })
}

impl<T> Sequence<T> {
    /// Reads the signed OCCT index overload; `sequence.value_signed(-1)` returns OutOfRange.
    pub fn value_signed(&self, index: i32) -> Result<&T, Failure> {
        self.value(unsigned_sequence_index("Value", index)?)
    }

    /// Mutably reads the signed overload; `*sequence.change_value_signed(1)? = value`.
    pub fn change_value_signed(&mut self, index: i32) -> Result<&mut T, Failure> {
        self.change_value(unsigned_sequence_index("ChangeValue", index)?)
    }

    /// Sets a signed one-based position; `sequence.set_value_signed(1, value)?`.
    pub fn set_value_signed(&mut self, index: i32, value: T) -> Result<(), Failure> {
        self.set_value(unsigned_sequence_index("SetValue", index)?, value)
    }

    /// Exchanges signed node positions; `sequence.exchange_signed(1, 2)?`.
    pub fn exchange_signed(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        let first = unsigned_sequence_index("Exchange", first)?;
        let second = unsigned_sequence_index("Exchange", second)?;
        self.exchange(first, second)
    }

    /// Removes a signed node position; `sequence.remove_signed(1)?`.
    pub fn remove_signed(&mut self, index: i32) -> Result<(), Failure> {
        self.remove(unsigned_sequence_index("Remove", index)?)
    }

    /// Removes an inclusive signed range; `sequence.remove_range_signed(1, 2)?`.
    pub fn remove_range_signed(&mut self, first: i32, last: i32) -> Result<(), Failure> {
        let first = unsigned_sequence_index("Remove", first)?;
        let last = unsigned_sequence_index("Remove", last)?;
        self.remove_range(first, last)
    }

    /// Inserts before a signed position; `sequence.insert_before_signed(1, value)?`.
    pub fn insert_before_signed(&mut self, index: i32, value: T) -> Result<&mut T, Failure> {
        self.insert_before(unsigned_sequence_index("InsertBefore", index)?, value)
    }

    /// Inserts after a signed position, zero prepending; `sequence.insert_after_signed(0, value)?`.
    pub fn insert_after_signed(&mut self, index: i32, value: T) -> Result<&mut T, Failure> {
        self.insert_after(unsigned_sequence_index("InsertAfter", index)?, value)
    }

    /// Constructs before a signed position after validation; `sequence.emplace_before_signed(1, || 4)?`.
    pub fn emplace_before_signed(
        &mut self,
        index: i32,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, Failure> {
        self.emplace_before(unsigned_sequence_index("EmplaceBefore", index)?, construct)
    }

    /// Constructs after a signed position after validation; `sequence.emplace_after_signed(0, || 4)?`.
    pub fn emplace_after_signed(
        &mut self,
        index: i32,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, Failure> {
        self.emplace_after(unsigned_sequence_index("EmplaceAfter", index)?, construct)
    }

    /// Splits at a signed index, preserving the destination on negative input; `sequence.split_signed(-1, &mut tail)`.
    pub fn split_signed(&mut self, index: i32, destination: &mut Self) -> Result<(), Failure> {
        self.split(unsigned_sequence_index("Split", index)?, destination)
    }
}

impl<T: Clone> Sequence<T> {
    /// Splices before a signed position, checking negatives even for empty sources; `sequence.insert_sequence_before_signed(1, &mut tail)?`.
    pub fn insert_sequence_before_signed(
        &mut self,
        index: i32,
        source: &mut Self,
    ) -> Result<(), Failure> {
        self.insert_sequence_before(unsigned_sequence_index("InsertBefore", index)?, source)
    }

    /// Splices after a signed position, checking negatives even for empty sources; `sequence.insert_sequence_after_signed(0, &mut tail)?`.
    pub fn insert_sequence_after_signed(
        &mut self,
        index: i32,
        source: &mut Self,
    ) -> Result<(), Failure> {
        self.insert_sequence_after(unsigned_sequence_index("InsertAfter", index)?, source)
    }
}
