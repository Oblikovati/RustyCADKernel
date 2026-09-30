// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_OBB.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `OBB_ExtremePointsSelector`: BVH descent that finds the extreme projections on an axis.
use crate::bnd::Range1;
use crate::bvh::{BvhBaseTraverse, BvhBoxSet, BvhSetTraverse, BvhTraverse, BvhVec3d};
use crate::gp::Xyz;
use rustycad_tkernel::standard::Failure;

/// The point set of the optimal DiTO path: one box of half size `tolerance` per point.
pub(super) type PointBoxSet = BvhBoxSet<BvhVec3d, Xyz>;

/// Keeps the smallest and largest `axis · point` seen so far and the points reaching them.
///
/// Ties keep the first point visited, so the traversal order decides which of several
/// equally extreme points is reported. Example: see `DitoBuilder::project`.
pub(super) struct ExtremePointsSelector<'s> {
    set: Option<&'s mut PointBoxSet>,
    axis: Xyz,
    minimum: f64,
    maximum: f64,
    minimum_point: Xyz,
    maximum_point: Xyz,
}

impl<'s> ExtremePointsSelector<'s> {
    /// Starts with the native `RealLast`/`RealFirst` sentinels (no parameter computed yet).
    pub(super) fn new(set: &'s mut PointBoxSet, axis: Xyz) -> Self {
        Self {
            set: Some(set),
            axis,
            minimum: f64::MAX,
            maximum: -f64::MAX,
            minimum_point: Xyz::default(),
            maximum_point: Xyz::default(),
        }
    }
    /// Minimal and maximal projection parameters with their points.
    pub(super) fn extremes(&self) -> super::ExtremePair {
        (
            (self.minimum, self.minimum_point),
            (self.maximum, self.maximum_point),
        )
    }
    fn nothing_computed(&self) -> bool {
        self.minimum > self.maximum
    }
}

impl BvhBaseTraverse<Range1> for ExtremePointsSelector<'_> {
    /// Prefers the branch whose projected range extends further beyond the current one.
    fn is_metric_better(&self, left: &Range1, right: &Range1) -> bool {
        if self.nothing_computed() {
            return true;
        }
        let (Some(left), Some(right)) = (left.get(), right.get()) else {
            return true;
        };
        self.extension(left) > self.extension(right)
    }
    /// Rejects a branch whose projected range lies strictly inside the current one.
    fn reject_metric(&self, metric: &Range1) -> bool {
        if self.nothing_computed() {
            return false;
        }
        let Some([low, high]) = metric.get() else {
            return false;
        };
        low > self.minimum && high < self.maximum
    }
}

impl ExtremePointsSelector<'_> {
    fn extension(&self, [low, high]: [f64; 2]) -> f64 {
        let mut extension = 0.0;
        if low < self.minimum {
            extension += self.minimum - low;
        }
        if high > self.maximum {
            extension += high - self.maximum;
        }
        extension
    }
}

impl BvhTraverse<BvhVec3d, Range1> for ExtremePointsSelector<'_> {
    /// Projects the eight node corners (x, then y, then z from min to max); the node is
    /// rejected when no corner widens the current range, and its metric is the widened range.
    fn reject_node(
        &self,
        corner_min: &BvhVec3d,
        corner_max: &BvhVec3d,
        metric: &mut Range1,
    ) -> bool {
        if self.nothing_computed() {
            return false;
        }
        let (mut low, mut high) = (self.minimum, self.maximum);
        let mut is_to_reject = true;
        for corner in 0..8 {
            let pick = |bit: usize, min: f64, max: f64| if corner & bit == 0 { min } else { max };
            let x = pick(4, corner_min.x, corner_max.x);
            let y = pick(2, corner_min.y, corner_max.y);
            let z = pick(1, corner_min.z, corner_max.z);
            let parameter = self.axis.dot(Xyz::new(x, y, z));
            if parameter < low {
                low = parameter;
                is_to_reject = false;
            } else if parameter > high {
                high = parameter;
                is_to_reject = false;
            }
        }
        // low <= minimum <= maximum <= high here, so the native `Last < First` raise of
        // Bnd_Range cannot fire (a NaN bound compares false and is stored as in C++).
        *metric = Range1::new(low, high).unwrap_or_default();
        is_to_reject
    }
    /// Takes the element's point as the new minimum and/or maximum on strict improvement.
    fn accept(&mut self, index: i32, _metric: &Range1) -> Result<bool, Failure> {
        let Some(set) = self.set.as_deref() else {
            return Ok(true);
        };
        let point = set.element(index)?;
        let parameter = self.axis.dot(point);
        if parameter < self.minimum {
            self.minimum = parameter;
            self.minimum_point = point;
        }
        if parameter > self.maximum {
            self.maximum = parameter;
            self.maximum_point = point;
        }
        Ok(true)
    }
}

impl BvhSetTraverse<BvhVec3d, Range1> for ExtremePointsSelector<'_> {
    type Set = PointBoxSet;
    fn bvh_set_mut(&mut self) -> Option<&mut PointBoxSet> {
        self.set.as_deref_mut()
    }
}
