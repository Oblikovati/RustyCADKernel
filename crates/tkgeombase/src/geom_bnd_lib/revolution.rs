// Rust translation of GeomBndLib_SurfaceOfRevolution at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_SurfaceOfRevolution`: the basis curve box revolved around the axis.
use super::live;
use super::swept::{
    BasisBox, basis_curve_box, clamped_basis_range, other_surface_box, revolution_box,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomSurfaceOfRevolution, GeomSweptSurface};
use rustycad_tkmath::bnd::BoundingBox3;

/// `GeomBndLib_SurfaceOfRevolution`: the box of a `Geom_SurfaceOfRevolution` patch.
pub struct SurfaceOfRevolution {
    geom: Handle<GeomSurfaceOfRevolution>,
}

// The V range on the basis: infinite ends replaced by the curve bounds, then clamped.
fn basis_range(curve: &dyn GeomCurve, (v_min, v_max): (f64, f64)) -> (f64, f64) {
    let first: f64 = if precision::is_negative_infinite(v_min) {
        curve.first_parameter()
    } else {
        v_min
    };
    let last: f64 = if precision::is_positive_infinite(v_max) {
        curve.last_parameter()
    } else {
        v_max
    };
    clamped_basis_range(curve, (first, last))
}

impl SurfaceOfRevolution {
    /// The evaluator of `surface`; `SurfaceOfRevolution::new(handle)`.
    pub fn new(surface: Handle<GeomSurfaceOfRevolution>) -> Self {
        Self { geom: surface }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomSurfaceOfRevolution> {
        &self.geom
    }
    fn surface(&self) -> Result<&GeomSurfaceOfRevolution, Failure> {
        live(&self.geom, "GeomBndLib_SurfaceOfRevolution")
    }
    /// `Box(Tol)`: the natural bounds; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let (u1, u2, v1, v2) = self.surface()?.bounds();
        self.bounding_box_range(u1, u2, v1, v2, tolerance)
    }
    /// `Box(UMin, UMax, VMin, VMax, Tol)`: the basis curve box of the V range (infinite
    /// ends replaced by the curve bounds, clamped on a non-periodic curve) with the
    /// revolution arc `[UMin, UMax]` of its eight corners; a void curve box or any raised
    /// exception falls back to GeomBndLib_OtherSurface on the original parameters;
    /// `evaluator.bounding_box_range(0.0, 6.3, 0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        self.guarded([u_min, u_max, v_min, v_max], tolerance, BasisBox::Box)
    }
    /// `BoxOptimal(UMin, UMax, VMin, VMax, Tol)`: as `Box` with the basis curve's optimal
    /// box and OtherSurface's optimal fallback;
    /// `evaluator.bounding_box_optimal_range(0.0, 6.3, 0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        self.guarded([u_min, u_max, v_min, v_max], tolerance, BasisBox::Optimal)
    }
    /// `BoxOptimal(Tol)`: the natural bounds; `evaluator.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let (u1, u2, v1, v2) = self.surface()?.bounds();
        self.bounding_box_optimal_range(u1, u2, v1, v2, tolerance)
    }
    // The native try/catch: any failure of the revolution runs the OtherSurface fallback.
    fn guarded(
        &self,
        patch: [f64; 4],
        tolerance: f64,
        kind: BasisBox,
    ) -> Result<BoundingBox3, Failure> {
        self.revolved(patch, tolerance, kind)
            .or_else(|_| other_surface_box(&self.geom, patch, tolerance, kind))
    }
    fn revolved(
        &self,
        patch: [f64; 4],
        tolerance: f64,
        kind: BasisBox,
    ) -> Result<BoundingBox3, Failure> {
        let [u_min, u_max, v_min, v_max] = patch;
        let surface = self.surface()?;
        let basis = surface.basis_curve();
        let axis = surface.axis();
        let range = basis_range(
            live(&basis, "GeomBndLib_SurfaceOfRevolution")?,
            (v_min, v_max),
        );
        let curve_box: BoundingBox3 = basis_curve_box(&basis, range, kind)?;
        if curve_box.is_void() {
            return other_surface_box(&self.geom, patch, tolerance, kind);
        }
        revolution_box(
            &curve_box,
            (axis.location(), axis.direction()),
            (u_min, u_max),
            tolerance,
        )
    }
}
