// IntTools_Range.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! A pair of parameter bounds, preserved in their given order.
/// `IntTools_Range`; `Range::new(3.0, 1.0).range() == (3.0, 1.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Range {
    first: f64,
    last: f64,
}
impl Range {
    /// Constructs the given bounds without sorting; `Range::new(0.0, 1.0)`.
    pub fn new(first: f64, last: f64) -> Self {
        Self { first, last }
    }
    /// Replaces the first bound; `range.set_first(2.0)`.
    pub fn set_first(&mut self, first: f64) {
        self.first = first;
    }
    /// Replaces the last bound; `range.set_last(4.0)`.
    pub fn set_last(&mut self, last: f64) {
        self.last = last;
    }
    /// Returns the first bound; `Range::default().first() == 0.0`.
    pub fn first(&self) -> f64 {
        self.first
    }
    /// Returns the last bound; `Range::default().last() == 0.0`.
    pub fn last(&self) -> f64 {
        self.last
    }
    /// Returns both bounds; `Range::new(1.0, 2.0).range() == (1.0, 2.0)`.
    pub fn range(&self) -> (f64, f64) {
        (self.first, self.last)
    }
}
