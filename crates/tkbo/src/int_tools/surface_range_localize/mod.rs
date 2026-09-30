// IntTools_SurfaceRangeLocalizeData at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Surface subdivision cache; copies share grid storage but retain independent frames.
use super::SurfaceRangeSample;
use rustycad_tkernel::{
    ncollection::{ArrayError, DataMap, HArray1, HArray2, HashKey, Map},
    standard::Handle,
};
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
mod frame;
mod grid;
impl HashKey for SurfaceRangeSample {
    fn occt_hash(&self) -> usize {
        self.hash_code()
    }
}
/// `IntTools_SurfaceRangeLocalizeData`; `SurfaceRangeLocalizeData::default()`.
pub struct SurfaceRangeLocalizeData {
    samples: [i32; 2],
    minimum: [f64; 2],
    excluded: Map<SurfaceRangeSample>,
    boxes: DataMap<SurfaceRangeSample, BoundingBox3>,
    parameters: [Handle<HArray1<f64>>; 2],
    points: Handle<HArray2<Point3>>,
    frame: [[i32; 2]; 2],
    deflection: f64,
}
impl Default for SurfaceRangeLocalizeData {
    fn default() -> Self {
        Self::new(1, 1, 0.0, 0.0)
    }
}
impl Clone for SurfaceRangeLocalizeData {
    fn clone(&self) -> Self {
        let mut result = Self::default();
        result.assign(self);
        result
    }
}
impl SurfaceRangeLocalizeData {
    /// Stores subdivision settings; `SurfaceRangeLocalizeData::new(2, 3, 0.1, 0.2)`.
    pub fn new(u: i32, v: i32, min_u: f64, min_v: f64) -> Self {
        Self {
            samples: [u, v],
            minimum: [min_u, min_v],
            excluded: Map::default(),
            boxes: DataMap::default(),
            parameters: [Handle::default(), Handle::default()],
            points: Handle::default(),
            frame: [[0; 2]; 2],
            deflection: 0.0,
        }
    }
    /// Copies maps and shares non-null grid handles; `target.assign(&source)`.
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        self.samples = other.samples;
        self.minimum = other.minimum;
        self.excluded.assign(&other.excluded);
        self.boxes.assign(&other.boxes);
        self.frame = other.frame;
        self.deflection = other.deflection;
        for axis in 0..2 {
            if !other.parameters[axis].is_null() {
                self.parameters[axis] = other.parameters[axis].clone();
            }
        }
        // Pinned Assign deliberately retains destination arrays when a source handle is null.
        if !other.points.is_null() {
            self.points = other.points.clone();
        }
        self
    }
    /// Marks exclusion without invalidating a box; `cache.add_out_range(&sample)`.
    pub fn add_out_range(&mut self, sample: &SurfaceRangeSample) {
        self.excluded.add(*sample);
    }
    /// Replaces a cached box; `cache.add_box(&sample, &bounds)`.
    pub fn add_box(&mut self, sample: &SurfaceRangeSample, bounds: &BoundingBox3) {
        self.boxes.bind(*sample, bounds.clone());
    }
    /// Leaves output unchanged on a miss; `cache.find_box(&sample, &mut bounds)`.
    pub fn find_box(&self, sample: &SurfaceRangeSample, bounds: &mut BoundingBox3) -> bool {
        self.boxes.find_into(sample, bounds)
    }
    /// Checks exclusion; `cache.is_range_out(&sample)`.
    pub fn is_range_out(&self, sample: &SurfaceRangeSample) -> bool {
        self.excluded.contains(sample)
    }
    /// Appends exclusions in native bucket order; `cache.list_range_out(&mut samples)`.
    pub fn list_range_out(&self, samples: &mut Vec<SurfaceRangeSample>) {
        samples.extend(self.excluded.iter().copied());
    }
    /// Clears exclusions while retaining boxes and grid; `cache.remove_range_out_all()`.
    pub fn remove_range_out_all(&mut self) {
        self.excluded.clear(true);
    }
    /// Reads the stored setting; `cache.get_nb_sample_u()`.
    pub fn get_nb_sample_u(&self) -> i32 {
        self.samples[0]
    }
    /// Reads the stored setting; `cache.get_nb_sample_v()`.
    pub fn get_nb_sample_v(&self) -> i32 {
        self.samples[1]
    }
    /// Reads the stored setting; `cache.get_min_range_u()`.
    pub fn get_min_range_u(&self) -> f64 {
        self.minimum[0]
    }
    /// Reads the stored setting; `cache.get_min_range_v()`.
    pub fn get_min_range_v(&self) -> f64 {
        self.minimum[1]
    }
    /// Reads the stored setting; `cache.get_grid_deflection()`.
    pub fn get_grid_deflection(&self) -> f64 {
        self.deflection
    }
    /// Sets the grid deflection independently; `cache.set_grid_deflection(0.1)`.
    pub fn set_grid_deflection(&mut self, value: f64) {
        self.deflection = value;
    }
    /// Releases grid handles and resets frame/deflection only; `cache.clear_grid()`.
    pub fn clear_grid(&mut self) {
        self.parameters = [Handle::default(), Handle::default()];
        self.points = Handle::default();
        self.frame = [[0; 2]; 2];
        self.deflection = 0.0;
    }
}
