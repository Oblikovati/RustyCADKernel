// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_OBB.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::OrientedBoundingBox;
use super::dito::DitoBuilder;
use crate::gp::{Direction3, Point3, Xyz};
use rustycad_tkernel::ncollection::{Array1, ArrayError};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

/// `NCollection_Array1` access failures surface as the native `Standard_OutOfRange`.
pub(super) fn out_of_range(error: ArrayError) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("Bnd_OBB::ReBuild: {error}"),
        "",
    )
}

impl OrientedBoundingBox {
    /// `ReBuild`: covers every point, enlarged by the per-point tolerances when given; one and
    /// two points have closed forms, more points run DiTO (`optimal` projects every point
    /// through a BVH and tries every extreme-point triangle).
    ///
    /// Tolerances are read at the point indices, so both arrays must share bounds. An empty
    /// list or mismatched bounds raise `Standard_OutOfRange` (the native `First()`/`Value()`
    /// read outside the array). Example: `obb.rebuild(&points, None, false)?`.
    pub fn rebuild(
        &mut self,
        points: &Array1<'_, Point3>,
        tolerances: Option<&Array1<'_, f64>>,
        optimal: bool,
    ) -> Result<(), Failure> {
        match points.len() {
            1 => self.rebuild_one(points, tolerances),
            2 => self.rebuild_two(points, tolerances),
            _ => DitoBuilder::new(points, tolerances, optimal)?.build_box(self),
        }
    }
    fn rebuild_one(
        &mut self,
        points: &Array1<'_, Point3>,
        tolerances: Option<&Array1<'_, f64>>,
    ) -> Result<(), Failure> {
        self.process_one_point(points.first().map_err(out_of_range)?.xyz());
        if let Some(tolerances) = tolerances {
            self.enlarge(*tolerances.first().map_err(out_of_range)?);
        }
        Ok(())
    }
    fn rebuild_two(
        &mut self,
        points: &Array1<'_, Point3>,
        tolerances: Option<&Array1<'_, f64>>,
    ) -> Result<(), Failure> {
        let (first_tolerance, last_tolerance) = match tolerances {
            None => (0.0, 0.0),
            Some(values) => (
                *values.first().map_err(out_of_range)?,
                *values.last().map_err(out_of_range)?,
            ),
        };
        let first = points.first().map_err(out_of_range)?.xyz();
        let last = points.last().map_err(out_of_range)?.xyz();
        self.segment_box(first, last, [first_tolerance, last_tolerance])
    }
    /// Box around a segment: X along the segment, Y orthogonal to X in the plane of X and
    /// its largest of |x|,|y| components, Z their normalized cross product.
    fn segment_box(&mut self, first: Xyz, last: Xyz, tolerances: [f64; 2]) -> Result<(), Failure> {
        let delta = last - first;
        let length = delta.modulus();
        self.is_aa_box = false;
        // std::max keeps the first operand unless it is less than the second.
        let radius = if tolerances[0] < tolerances[1] {
            tolerances[1]
        } else {
            tolerances[0]
        };
        self.half_sizes[1] = radius;
        self.half_sizes[2] = radius;
        if length < precision::confusion() {
            self.process_one_point(first);
            self.enlarge(self.half_sizes[1] + precision::confusion());
            return Ok(());
        }
        self.half_sizes[0] = 0.5 * (length + tolerances[0] + tolerances[1]);
        self.axes[0] = delta / length;
        let x = self.axes[0];
        self.axes[1] = if x.x().abs() > x.y().abs() {
            Xyz::new(-x.z(), 0.0, x.x())
        } else {
            Xyz::new(0.0, -x.z(), x.y())
        };
        self.axes[2] = x.crossed(self.axes[1]).normalized()?;
        self.center = first + 0.5 * (length - tolerances[0] + tolerances[1]) * x;
        Ok(())
    }
    /// `Add(gp_Pnt)`: a void box becomes the point; otherwise rebuilds from the eight
    /// vertices and the point. Example: `obb.add_point(Point3::new(1.0, 2.0, 3.0))?`.
    pub fn add_point(&mut self, point: Point3) -> Result<(), Failure> {
        let Some(vertices) = self.vertices() else {
            self.center = point.xyz();
            self.axes = [Direction3::X, Direction3::Y, Direction3::Z].map(Xyz::from);
            self.half_sizes = [0.0; 3];
            self.is_aa_box = true;
            return Ok(());
        };
        let mut list: [Point3; 9] = [point; 9];
        list[..8].copy_from_slice(&vertices);
        self.rebuild(&Array1::from_shared(&list, 0), None, false)
    }
    /// `Add(Bnd_OBB)`: ignores a void box, copies into a void box, otherwise rebuilds from
    /// both vertex sets. Example: `obb.add_box(&other)?`.
    pub fn add_box(&mut self, other: &Self) -> Result<(), Failure> {
        let Some(theirs) = other.vertices() else {
            return Ok(());
        };
        let Some(mine) = self.vertices() else {
            *self = *other;
            return Ok(());
        };
        let mut list = [Point3::default(); 16];
        list[..8].copy_from_slice(&mine);
        list[8..].copy_from_slice(&theirs);
        self.rebuild(&Array1::from_shared(&list, 0), None, false)
    }
}
