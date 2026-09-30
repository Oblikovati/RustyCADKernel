// IntTools_CurveRangeLocalizeData.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Curve subdivision cache with native map traversal and independent exclusion markers.
use super::CurveRangeSample;
use rustycad_tkernel::ncollection::{DataMap, HashKey, Map};
use rustycad_tkmath::bnd::BoundingBox3;
impl HashKey for CurveRangeSample {
    fn occt_hash(&self) -> usize {
        self.hash_code()
    }
}
/// `IntTools_CurveRangeLocalizeData`; `CurveRangeLocalizeData::new(4, 1e-7)`.
pub struct CurveRangeLocalizeData {
    samples: i32,
    min_range: f64,
    excluded: Map<CurveRangeSample>,
    boxes: DataMap<CurveRangeSample, BoundingBox3>,
}
impl CurveRangeLocalizeData {
    /// Stores subdivision settings without validation; `CurveRangeLocalizeData::new(4, 1e-7)`.
    pub fn new(samples: i32, min_range: f64) -> Self {
        Self {
            samples,
            min_range,
            excluded: Map::default(),
            boxes: DataMap::default(),
        }
    }
    /// Reads the subdivision count; `cache.get_nb_sample()`.
    pub fn get_nb_sample(&self) -> i32 {
        self.samples
    }
    /// Reads the minimum interval length; `cache.get_min_range()`.
    pub fn get_min_range(&self) -> f64 {
        self.min_range
    }
    /// Marks exclusion and removes any cached box; `cache.add_out_range(&sample)`.
    pub fn add_out_range(&mut self, sample: &CurveRangeSample) {
        self.excluded.add(*sample);
        self.boxes.unbind(sample);
    }
    /// Replaces a cached box without removing exclusion; `cache.add_box(&sample, &bounds)`.
    pub fn add_box(&mut self, sample: &CurveRangeSample, bounds: &BoundingBox3) {
        self.boxes.bind(*sample, bounds.clone());
    }
    /// Copies a box only when found; `cache.find_box(&sample, &mut bounds)`.
    pub fn find_box(&self, sample: &CurveRangeSample, bounds: &mut BoundingBox3) -> bool {
        self.boxes.find_into(sample, bounds)
    }
    /// Checks the independent exclusion set; `cache.is_range_out(&sample)`.
    pub fn is_range_out(&self, sample: &CurveRangeSample) -> bool {
        self.excluded.contains(sample)
    }
    /// Appends in native bucket order without clearing the target; `cache.list_range_out(&mut samples)`.
    pub fn list_range_out(&self, samples: &mut Vec<CurveRangeSample>) {
        samples.extend(self.excluded.iter().copied());
    }
}
