// IntTools_MarkedRangeSet.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Flagged adjacent intervals. Duplicate bounds and native insertion ordering are preserved.
mod insert;
mod queries;
use super::Range;
use rustycad_tkernel::standard::Failure;
/// `IntTools_MarkedRangeSet`; `MarkedRangeSet::new(0.0, 1.0, 0)`.
#[derive(Clone, Debug, Default)]
pub struct MarkedRangeSet {
    bounds: Vec<f64>,
    flags: Vec<i32>,
    range_number: i32,
    found_indices: Vec<i32>,
}
impl MarkedRangeSet {
    /// Initializes one flagged range; `MarkedRangeSet::new(0.0, 1.0, 3)`.
    pub fn new(first: f64, last: f64, flag: i32) -> Self {
        let mut result = Self::default();
        result.set_boundaries(first, last, flag);
        result
    }
    /// Copies bounds without sorting; `MarkedRangeSet::from_ranges(&[0.0, 1.0, 2.0], 3)`.
    pub fn from_ranges(bounds: &[f64], flag: i32) -> Self {
        let mut result = Self::default();
        result.set_ranges(bounds, flag);
        result
    }
    /// Replaces the set with one interval; `set.set_boundaries(-1.0, 2.0, 4)`.
    pub fn set_boundaries(&mut self, first: f64, last: f64, flag: i32) {
        self.bounds = vec![first, last];
        self.flags = vec![flag];
        self.range_number = 1;
    }
    /// Copies the supplied order; `set.set_ranges(&[0.0, 0.5, 1.0], 2)`.
    pub fn set_ranges(&mut self, bounds: &[f64], flag: i32) {
        self.bounds = bounds.to_vec();
        self.range_number = self.bounds.len() as i32 - 1;
        self.flags = vec![flag; self.range_number.max(0) as usize];
    }
    /// Reports the native interval count; `MarkedRangeSet::default().length() == 0`.
    pub fn length(&self) -> i32 {
        self.range_number
    }
    /// Replaces a one-based interval flag; `set.set_flag(1, 9)?`.
    pub fn set_flag(&mut self, index: i32, flag: i32) -> Result<(), Failure> {
        let slot = checked_slot(index, self.flags.len())?;
        self.flags[slot] = flag;
        Ok(())
    }
    /// Reads a one-based interval flag; `set.flag(1)?`.
    pub fn flag(&self, index: i32) -> Result<i32, Failure> {
        Ok(self.flags[checked_slot(index, self.flags.len())?])
    }
    /// Reads a one-based interval; `set.range(1)?.range()`.
    pub fn range(&self, index: i32) -> Result<Range, Failure> {
        Ok(Range::new(self.bound(index)?, self.bound(index + 1)?))
    }
    fn bound(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.bounds[checked_slot(index, self.bounds.len())?])
    }
}
fn checked_slot(index: i32, length: usize) -> Result<usize, Failure> {
    if index > 0 && index as usize <= length {
        return Ok(index as usize - 1);
    }
    Err(Failure::with_type(
        "Standard_OutOfRange",
        &format!("IntTools_MarkedRangeSet index {index}; expected 1..={length}"),
        "",
    ))
}
