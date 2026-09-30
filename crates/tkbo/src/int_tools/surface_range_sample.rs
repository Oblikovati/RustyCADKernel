// IntTools_SurfaceRangeSample.hxx/.cxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Independent subdivisions of a surface's U and V parameter intervals.
use super::{CurveRangeSample, Range};
use rustycad_tkernel::standard::hash_bytes;
/// `IntTools_SurfaceRangeSample`; `SurfaceRangeSample::new(1, 2, 3, 4)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceRangeSample {
    u: CurveRangeSample,
    v: CurveRangeSample,
}
impl SurfaceRangeSample {
    /// Stores both indices and depths; `SurfaceRangeSample::new(1, 2, 3, 4)`.
    pub fn new(index_u: i32, depth_u: i32, index_v: i32, depth_v: i32) -> Self {
        let mut result = Self::default();
        result.set_indexes(index_u, index_v);
        result.set_depth_u(depth_u);
        result.set_depth_v(depth_v);
        result
    }
    /// Copies curve samples; `SurfaceRangeSample::from_ranges(u, v)`.
    pub fn from_ranges(u: CurveRangeSample, v: CurveRangeSample) -> Self {
        Self { u, v }
    }
    /// Copies both samples and returns self; `sample.assign(&other)`.
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        *self = *other;
        self
    }
    /// Replaces both samples; `sample.set_ranges(u, v)`.
    pub fn set_ranges(&mut self, u: CurveRangeSample, v: CurveRangeSample) {
        self.u = u;
        self.v = v;
    }
    /// Copies both samples; `sample.get_ranges()`.
    pub fn get_ranges(&self) -> (CurveRangeSample, CurveRangeSample) {
        (self.u, self.v)
    }
    /// Replaces both indices; `sample.set_indexes(1, 2)`.
    pub fn set_indexes(&mut self, u: i32, v: i32) {
        self.u.set_range_index(u);
        self.v.set_range_index(v);
    }
    /// Returns both indices; `sample.get_indexes()`.
    pub fn get_indexes(&self) -> (i32, i32) {
        (self.u.get_range_index(), self.v.get_range_index())
    }
    /// Returns both depths; `sample.get_depths()`.
    pub fn get_depths(&self) -> (i32, i32) {
        (self.u.get_depth(), self.v.get_depth())
    }
    /// Replaces the U sample; `sample.set_sample_range_u(curve_sample)`.
    pub fn set_sample_range_u(&mut self, sample: CurveRangeSample) {
        self.u = sample;
    }
    /// Borrows the U sample; `sample.get_sample_range_u()`.
    pub fn get_sample_range_u(&self) -> &CurveRangeSample {
        &self.u
    }
    /// Replaces the U index; `sample.set_index_u(2)`.
    pub fn set_index_u(&mut self, index: i32) {
        self.u.set_range_index(index);
    }
    /// Returns the U index; `sample.get_index_u()`.
    pub fn get_index_u(&self) -> i32 {
        self.u.get_range_index()
    }
    /// Replaces the U depth; `sample.set_depth_u(3)`.
    pub fn set_depth_u(&mut self, depth: i32) {
        self.u.set_depth(depth);
    }
    /// Returns the U depth; `sample.get_depth_u()`.
    pub fn get_depth_u(&self) -> i32 {
        self.u.get_depth()
    }
    /// Computes the U parameter interval; `sample.get_range_u(0.0, 1.0, 2)`.
    pub fn get_range_u(&self, first: f64, last: f64, samples: i32) -> Range {
        self.u.get_range(first, last, samples)
    }
    /// Returns the first U child index; `sample.get_range_index_u_deeper(2)`.
    pub fn get_range_index_u_deeper(&self, samples: i32) -> i32 {
        self.u.get_range_index_deeper(samples)
    }
    /// Replaces the V sample; `sample.set_sample_range_v(curve_sample)`.
    pub fn set_sample_range_v(&mut self, sample: CurveRangeSample) {
        self.v = sample;
    }
    /// Borrows the V sample; `sample.get_sample_range_v()`.
    pub fn get_sample_range_v(&self) -> &CurveRangeSample {
        &self.v
    }
    /// Replaces the V index; `sample.set_index_v(2)`.
    pub fn set_index_v(&mut self, index: i32) {
        self.v.set_range_index(index);
    }
    /// Returns the V index; `sample.get_index_v()`.
    pub fn get_index_v(&self) -> i32 {
        self.v.get_range_index()
    }
    /// Replaces the V depth; `sample.set_depth_v(3)`.
    pub fn set_depth_v(&mut self, depth: i32) {
        self.v.set_depth(depth);
    }
    /// Returns the V depth; `sample.get_depth_v()`.
    pub fn get_depth_v(&self) -> i32 {
        self.v.get_depth()
    }
    /// Computes the V parameter interval; `sample.get_range_v(0.0, 1.0, 2)`.
    pub fn get_range_v(&self, first: f64, last: f64, samples: i32) -> Range {
        self.v.get_range(first, last, samples)
    }
    /// Returns the first V child index; `sample.get_range_index_v_deeper(2)`.
    pub fn get_range_index_v_deeper(&self, samples: i32) -> i32 {
        self.v.get_range_index_deeper(samples)
    }
    /// Compares both samples; `sample.is_equal(&sample)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self == other
    }
    /// Hashes the native size_t pair of curve hashes; `sample.hash_code()`.
    pub fn hash_code(&self) -> usize {
        let bytes: Vec<u8> = [self.u.hash_code(), self.v.hash_code()]
            .into_iter()
            .flat_map(usize::to_ne_bytes)
            .collect();
        hash_bytes(&bytes)
    }
}
impl std::hash::Hash for SurfaceRangeSample {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash_code());
    }
}
