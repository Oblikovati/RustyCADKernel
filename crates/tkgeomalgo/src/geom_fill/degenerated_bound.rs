// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_DegeneratedBound.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_DegeneratedBound`: a boundary reduced to a point.
use super::{GeomFillBoundary, GeomFillBoundaryTolerances};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Point3, Vector3};

/// A boundary reduced to one point over a parameter range
/// (`GeomFill_DegeneratedBound`).
///
/// `let bound = GeomFillDegeneratedBound::new(point, 0.0, 1.0, 1e-4, 10.0); bound.is_degenerated()`
pub struct GeomFillDegeneratedBound {
    parent: StandardTransient,
    tolerances: GeomFillBoundaryTolerances,
    my_point: Point3,
    range: SharedState<(f64, f64)>,
}
impl_transient!(
    GeomFillDegeneratedBound,
    "GeomFill_DegeneratedBound",
    dyn GeomFillBoundary,
    parent,
    interfaces: [dyn GeomFillBoundary]
);

impl GeomFillDegeneratedBound {
    /// `GeomFill_DegeneratedBound(Point, First, Last, Tol3d, Tolang)`;
    /// `GeomFillDegeneratedBound::new(point, 0.0, 1.0, 1e-4, 10.0)`.
    pub fn new(point: Point3, first: f64, last: f64, tol3d: f64, tolang: f64) -> Self {
        Self {
            parent: StandardTransient::default(),
            tolerances: GeomFillBoundaryTolerances::new(tol3d, tolang),
            my_point: point,
            range: SharedState::new((first, last)),
        }
    }
}

impl GeomFillBoundary for GeomFillDegeneratedBound {
    /// The point, whatever `U`.
    fn value(&self, _u: f64) -> Result<Point3, Failure> {
        Ok(self.my_point)
    }
    /// The point and a null derivative.
    fn d1(&self, _u: f64) -> Result<(Point3, Vector3), Failure> {
        Ok((self.my_point, Vector3::new(0., 0., 0.)))
    }
    /// Only the range changes.
    fn reparametrize(
        &self,
        first: f64,
        last: f64,
        _has_df: bool,
        _has_dl: bool,
        _df: f64,
        _dl: f64,
        _rev: bool,
    ) -> Result<(), Failure> {
        *self.range.write() = (first, last);
        Ok(())
    }
    fn bounds(&self) -> Result<(f64, f64), Failure> {
        Ok(*self.range.read())
    }
    fn is_degenerated(&self) -> bool {
        true
    }
    fn tolerances(&self) -> &GeomFillBoundaryTolerances {
        &self.tolerances
    }
}
