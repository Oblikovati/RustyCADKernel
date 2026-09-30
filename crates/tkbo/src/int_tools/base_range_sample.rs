// IntTools_BaseRangeSample.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Subdivision depth shared by range samples.
/// `IntTools_BaseRangeSample`; `BaseRangeSample::new(2).get_depth() == 2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BaseRangeSample {
    depth: i32,
}
impl BaseRangeSample {
    /// Stores any depth, including negative values; `BaseRangeSample::new(-1)`.
    pub fn new(depth: i32) -> Self {
        Self { depth }
    }
    /// Changes subdivision depth; `sample.set_depth(4)`.
    pub fn set_depth(&mut self, depth: i32) {
        self.depth = depth;
    }
    /// Returns subdivision depth; `BaseRangeSample::default().get_depth() == 0`.
    pub fn get_depth(&self) -> i32 {
        self.depth
    }
}
