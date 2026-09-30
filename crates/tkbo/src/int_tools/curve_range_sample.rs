// IntTools_CurveRangeSample.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! An indexed subdivision of a curve's parameter interval.
use super::{BaseRangeSample, Range};
use rustycad_tkernel::standard::hash_bytes;
/// `IntTools_CurveRangeSample`; `CurveRangeSample::new(3).get_range_index() == 3`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CurveRangeSample {
    base: BaseRangeSample,
    index: i32,
}
impl CurveRangeSample {
    /// Stores the index at depth zero; `CurveRangeSample::new(3)`.
    pub fn new(index: i32) -> Self {
        Self {
            base: BaseRangeSample::default(),
            index,
        }
    }
    /// Changes the cell index; `sample.set_range_index(4)`.
    pub fn set_range_index(&mut self, index: i32) {
        self.index = index;
    }
    /// Reads the cell index; `CurveRangeSample::default().get_range_index() == 0`.
    pub fn get_range_index(&self) -> i32 {
        self.index
    }
    /// Compares depth and index; `sample.is_equal(&sample)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self == other
    }
    /// Subdivides using native `pow(samples, depth)`; `sample.get_range(0.0, 1.0, 2)`.
    pub fn get_range(&self, first: f64, last: f64, samples: i32) -> Range {
        if self.get_depth() <= 0 {
            return Range::new(first, last);
        }
        let divisor = f64::from(samples).powf(f64::from(self.get_depth()));
        let width = (last - first) / divisor;
        let start = first + f64::from(self.index) * width;
        Range::new(start, start + width)
    }
    /// Returns the first child's index; `CurveRangeSample::new(3).get_range_index_deeper(2) == 6`.
    pub fn get_range_index_deeper(&self, samples: i32) -> i32 {
        self.index.wrapping_mul(samples)
    }
    /// Native byte hash of depth and index; `sample.hash_code()`.
    pub fn hash_code(&self) -> usize {
        let mut bytes = [0u8; 8];
        bytes[..4].copy_from_slice(&self.get_depth().to_ne_bytes());
        bytes[4..].copy_from_slice(&self.index.to_ne_bytes());
        hash_bytes(&bytes)
    }
}
impl std::ops::Deref for CurveRangeSample {
    type Target = BaseRangeSample;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for CurveRangeSample {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl std::hash::Hash for CurveRangeSample {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash_code());
    }
}
