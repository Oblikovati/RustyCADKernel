// BOPDS_IndexRange.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! An inclusive range of shape indices, retaining the supplied order.
/// `BOPDS_IndexRange`; `IndexRange::new(1, 3).contains(2)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IndexRange {
    first: i32,
    last: i32,
}
impl IndexRange {
    /// Stores both indices; `IndexRange::new(2, 7)`.
    pub fn new(first: i32, last: i32) -> Self {
        Self { first, last }
    }
    /// Replaces the first index; `range.set_first(2)`.
    pub fn set_first(&mut self, first: i32) {
        self.first = first;
    }
    /// Replaces the last index; `range.set_last(7)`.
    pub fn set_last(&mut self, last: i32) {
        self.last = last;
    }
    /// Returns the first index; `IndexRange::default().first() == 0`.
    pub fn first(&self) -> i32 {
        self.first
    }
    /// Returns the last index; `IndexRange::default().last() == 0`.
    pub fn last(&self) -> i32 {
        self.last
    }
    /// Replaces both indices; `range.set_indices(2, 7)`.
    pub fn set_indices(&mut self, first: i32, last: i32) {
        self.first = first;
        self.last = last;
    }
    /// Reads both indices; `IndexRange::new(2, 7).indices() == (2, 7)`.
    pub fn indices(&self) -> (i32, i32) {
        (self.first, self.last)
    }
    /// Includes both endpoints, without normalizing reversed bounds; `range.contains(3)`.
    pub fn contains(&self, index: i32) -> bool {
        index >= self.first && index <= self.last
    }
    /// Writes native Dump text to an injected writer; `range.dump(&mut String::new())?`.
    pub fn dump(&self, out: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(out, " range: {} {}", self.first, self.last)
    }
}
