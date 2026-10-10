//! Native candidate-ranking predicates, which do not define a total ordering.
use crate::select3d::Select3DSensitiveEntity;
use rustycad_tkernel::{
    ncollection::Vec3,
    precision::confusion,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Direction3, Point3};

/// Candidate values used by the viewer selector; `SelectMgrSortCriterion::new()`.
#[derive(Clone, Default)]
pub struct SelectMgrSortCriterion {
    /// Detected entity; `criterion.entity.clone()`.
    pub entity: Handle<dyn Select3DSensitiveEntity>,
    /// Detected world point; `criterion.point`.
    pub point: Point3,
    /// Surface normal rounded to native single precision; `criterion.normal`.
    pub normal: Vec3<f32>,
    /// Distance from the view plane; `criterion.depth`.
    pub depth: f64,
    /// Distance on the view plane from the clicked point; `criterion.min_dist`.
    pub min_dist: f64,
    /// Selection tolerance; `criterion.tolerance`.
    pub tolerance: f64,
    /// Sensitive selection priority; `criterion.selection_priority`.
    pub selection_priority: i32,
    /// Display priority; `criterion.display_priority`.
    pub display_priority: i32,
    /// Z-layer ordering takes precedence over depth; `criterion.z_layer_position`.
    pub z_layer_position: i32,
    /// Count of entities for this owner; `criterion.nb_owner_matches`.
    pub nb_owner_matches: i32,
    /// Stored preference for the caller; `criterion.is_prefer_priority`.
    pub is_prefer_priority: bool,
}
impl SelectMgrSortCriterion {
    /// Create a null entity with zero values; `SelectMgrSortCriterion::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Compare depth, tolerance and priorities in native order; `criterion.is_closer_depth(&other)?`.
    pub fn is_closer_depth(&self, other: &Self) -> Result<bool, Failure> {
        if self.z_layer_position != other.z_layer_position {
            return Ok(self.z_layer_position > other.z_layer_position);
        }
        if (self.depth - other.depth).abs() > self.tolerance + other.tolerance {
            return Ok(self.depth < other.depth);
        }
        let cosine = self.normal_angle_cosine(other)?;
        let first = self.depth - self.tolerance;
        let second = other.depth - other.tolerance;
        if (first - second).abs() > confusion() && self.compare_adjusted_depth(other, cosine) {
            return Ok(first < second);
        }
        Ok(self.selection_priority > other.selection_priority
            || self.display_priority > other.display_priority
            || (self.selection_priority == other.selection_priority
                && self.min_dist < other.min_dist))
    }
    fn normal_angle_cosine(&self, other: &Self) -> Result<f64, Failure> {
        // OCCT 3d097a0 tests f32 modulus before constructing f64 directions.
        if self.normal.modulus() > 0. && other.normal.modulus() > 0. {
            let first = criterion_normal_direction(self.normal)?;
            let second = criterion_normal_direction(other.normal)?;
            return Ok(first.angle(&second).cos().abs());
        }
        Ok(1.)
    }
    fn compare_adjusted_depth(&self, other: &Self, cosine: f64) -> bool {
        (cosine > 0.5 && (self.tolerance - other.tolerance).abs() < confusion())
            || (cosine - 1.).abs() < confusion()
    }
    /// Apply the older priority-first predicate; `criterion.is_higher_priority(&other)`.
    pub fn is_higher_priority(&self, other: &Self) -> bool {
        if self.z_layer_position != other.z_layer_position {
            return self.z_layer_position > other.z_layer_position;
        }
        if self.selection_priority != other.selection_priority {
            return self.selection_priority > other.selection_priority;
        }
        if self.display_priority != other.display_priority {
            return self.display_priority > other.display_priority;
        }
        if (self.depth - other.depth).abs() <= confusion() {
            return self.min_dist < other.min_dist;
        }
        self.depth < other.depth
    }
}
fn criterion_normal_direction(normal: Vec3<f32>) -> Result<Direction3, Failure> {
    Direction3::new(
        f64::from(normal.x),
        f64::from(normal.y),
        f64::from(normal.z),
    )
}
