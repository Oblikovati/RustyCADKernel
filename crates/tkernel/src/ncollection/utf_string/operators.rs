//! Comparison and concatenation operators of NCollection_UtfString.
use super::UtfString;
use crate::ncollection::UtfUnit;
use std::ops::{Add, AddAssign};

/// Native `operator==`/`operator!=`: `IsEqual`, byte-wise over the buffers.
impl<T: UtfUnit> PartialEq for UtfString<T> {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl<T: UtfUnit> Eq for UtfString<T> {}

impl<T: UtfUnit> UtfString<T> {
    /// Units of `left` then `right`, with lengths summed as given (no rescan).
    fn joined(left: &Self, right: &Self, length: i32) -> Self {
        let mut text = Self::with_size(left.size + right.size, length);
        let left_count = left.size as usize / std::mem::size_of::<T>();
        let right_count = right.size as usize / std::mem::size_of::<T>();
        text.units[..left_count].copy_from_slice(&left.units[..left_count]);
        text.units[left_count..left_count + right_count]
            .copy_from_slice(&right.units[..right_count]);
        text
    }
}

/// Native `operator+=`. Quirk preserved: an empty right side (first unit zero) is skipped
/// entirely, so a length that counted only dropped illegal symbols is not added.
impl<T: UtfUnit> AddAssign<&UtfString<T>> for UtfString<T> {
    fn add_assign(&mut self, append: &UtfString<T>) {
        if append.is_empty() {
            return;
        }
        *self = Self::joined(self, append, self.length + append.length);
    }
}

/// Native friend `operator+`: always sums both cached lengths, even for empty operands.
impl<T: UtfUnit> Add<&UtfString<T>> for &UtfString<T> {
    type Output = UtfString<T>;
    fn add(self, right: &UtfString<T>) -> UtfString<T> {
        UtfString::joined(self, right, self.length + right.length)
    }
}

impl<T: UtfUnit> UtfString<T> {
    /// Returns the native compound-assignment destination; `text.append_returning(&suffix)`.
    pub fn append_returning(&mut self, append: &Self) -> &mut Self {
        *self += append;
        self
    }
    /// Appends this same string while its old buffer remains live; `text.append_self()`.
    pub fn append_self(&mut self) -> &mut Self {
        if !self.is_empty() {
            *self = Self::joined(self, self, self.length + self.length);
        }
        self
    }
}
