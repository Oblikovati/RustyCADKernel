// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_OBB.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `OBBTool`: the DiTO (ditetrahedron) OBB construction of Larsson and Källberg.
use super::accumulate::out_of_range;
use crate::bvh::{BvhBox, BvhLinearBuilder, BvhSetTraverse, BvhVec3d, constants::MAX_TREE_DEPTH};
use crate::gp::{Point3, Xyz};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use selector::{ExtremePointsSelector, PointBoxSet};
use std::sync::Arc;

mod build;
mod selector;
mod triangles;

/// Number of initial axes (the 7-axis set: 3 coordinate axes and 4 cube diagonals).
const INITIAL_AXES: usize = 7;
/// Number of extremal points: a minimum and a maximum per initial axis.
const EXTREMAL_POINTS: usize = 2 * INITIAL_AXES;
/// `INT_MAX` marker of a triangle index that was not found.
const UNSET: usize = usize::MAX;
/// Precomputed sqrt(3) of the native source.
const SQRT_3: f64 = 1.732_050_807_568_877_2;
/// Leaf size of the linear BVH over the points in the optimal mode.
const LEAF_SIZE: i32 = 30;
/// The minimal and maximal projection parameters on an axis, each with its point.
type ExtremePair = ((f64, Xyz), (f64, Xyz));

/// Cached projection interval on the axis joining two extremal points; void (min > max)
/// until computed.
#[derive(Clone, Copy)]
struct AxisInterval {
    minimum: f64,
    maximum: f64,
}
const UNCOMPUTED: AxisInterval = AxisInterval {
    minimum: f64::MAX,
    maximum: -f64::MAX,
};

/// DiTO state over a borrowed point list and optional per-point tolerances.
///
/// Example: `DitoBuilder::new(&points, None, false)?.build_box(&mut obb)?`.
pub(super) struct DitoBuilder<'p, 'a> {
    points: &'p Array1<'a, Point3>,
    tolerances: Option<&'p Array1<'a, f64>>,
    triangle: [usize; 5],
    extremal: [Xyz; EXTREMAL_POINTS],
    axes: [Xyz; 3],
    quality: f64,
    optimal: bool,
    point_set: Option<PointBoxSet>,
    intervals: [[AxisInterval; EXTREMAL_POINTS]; EXTREMAL_POINTS],
}

impl<'p, 'a> DitoBuilder<'p, 'a> {
    /// Builds the point BVH in the optimal mode, then finds the extremal points.
    pub(super) fn new(
        points: &'p Array1<'a, Point3>,
        tolerances: Option<&'p Array1<'a, f64>>,
        optimal: bool,
    ) -> Result<Self, Failure> {
        let mut builder = Self {
            points,
            tolerances,
            triangle: [0; 5],
            extremal: [Xyz::default(); EXTREMAL_POINTS],
            axes: [Xyz::default(); 3],
            quality: f64::MAX,
            optimal,
            point_set: None,
            intervals: [[UNCOMPUTED; EXTREMAL_POINTS]; EXTREMAL_POINTS],
        };
        if optimal {
            builder.point_set = Some(builder.point_box_set()?);
        }
        builder.compute_extreme_points()?;
        Ok(builder)
    }
    /// One box of half size `tolerance` (default `Precision::Confusion()`) per point, in a
    /// linear BVH with 30 elements per leaf.
    fn point_box_set(&self) -> Result<PointBoxSet, Failure> {
        let builder = Arc::new(BvhLinearBuilder::<BvhVec3d>::new(LEAF_SIZE, MAX_TREE_DEPTH));
        let mut set = PointBoxSet::new(Some(builder));
        set.set_size(self.points.len());
        for index in self.points.lower()..=self.points.upper() {
            let point = *self.points.value(index).map_err(out_of_range)?;
            let tolerance = match self.tolerances {
                Some(values) => *values.value(index).map_err(out_of_range)?,
                None => precision::confusion(),
            };
            let [x, y, z] = *point.coordinates();
            let bounds = BvhBox::from_corners(
                BvhVec3d::new(x - tolerance, y - tolerance, z - tolerance),
                BvhVec3d::new(x + tolerance, y + tolerance, z + tolerance),
            );
            set.add(point.xyz(), bounds);
        }
        set.build()?;
        Ok(set)
    }
    /// Projects on the initial axes to find the extremal points; the non-optimal mode also
    /// seeds the triangle with the most distant extremal pair and the point farthest from it.
    fn compute_extreme_points(&mut self) -> Result<(), Failure> {
        let diagonal = |y: f64, z: f64| Xyz::new(1.0, y, z) / SQRT_3;
        let initial_axes = [
            Xyz::new(1.0, 0.0, 0.0),
            Xyz::new(0.0, 1.0, 0.0),
            Xyz::new(0.0, 0.0, 1.0),
            diagonal(1.0, 1.0),
            diagonal(1.0, -1.0),
            diagonal(-1.0, 1.0),
            diagonal(-1.0, -1.0),
        ];
        for (index, axis) in initial_axes.into_iter().enumerate() {
            let ((_, low), (_, high)) = self.project(axis)?;
            self.extremal[2 * index] = low;
            self.extremal[2 * index + 1] = high;
        }
        if !self.optimal {
            self.triangle = [UNSET; 5];
            self.fill_most_distant_pair();
            self.fill_to_triangle3();
        }
        Ok(())
    }
    /// `myTriIdx[0..2]`: the extremal min/max pair with the largest square distance.
    fn fill_most_distant_pair(&mut self) {
        let mut max_square_distance = -1.0;
        for first in (0..EXTREMAL_POINTS).step_by(2) {
            let square_distance = Point3::from(self.extremal[first])
                .square_distance(&Point3::from(self.extremal[first + 1]));
            if square_distance > max_square_distance {
                max_square_distance = square_distance;
                self.triangle[0] = first;
                self.triangle[1] = first + 1;
            }
        }
    }
    /// `FindMinMax`: extremes of all points (optimal) or of the extremal points only.
    fn find_min_max(&mut self, axis: Xyz) -> Result<(f64, f64), Failure> {
        if self.optimal {
            let ((low, _), (high, _)) = self.project(axis)?;
            return Ok((low, high));
        }
        let (mut low, mut high) = (f64::MAX, -f64::MAX);
        for point in &self.extremal {
            let parameter = axis.dot(*point);
            if parameter < low {
                low = parameter;
            }
            if parameter > high {
                high = parameter;
            }
        }
        Ok((low, high))
    }
    /// `Project`: extremes of all points and the first points reaching them; through the
    /// BVH in the optimal mode, otherwise in list order.
    fn project(&mut self, axis: Xyz) -> Result<ExtremePair, Failure> {
        if let Some(set) = self.point_set.as_mut() {
            let mut selector = ExtremePointsSelector::new(set, axis);
            selector.select()?;
            return Ok(selector.extremes());
        }
        let mut low = (f64::MAX, Xyz::default());
        let mut high = (-f64::MAX, Xyz::default());
        for point in self.points.iter() {
            let parameter = axis.dot(point.xyz());
            if parameter < low.0 {
                low = (parameter, point.xyz());
            }
            if parameter > high.0 {
                high = (parameter, point.xyz());
            }
        }
        Ok((low, high))
    }
}
