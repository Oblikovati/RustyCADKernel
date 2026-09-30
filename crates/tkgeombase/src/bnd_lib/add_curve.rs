// Rust translation of BndLib_Add3dCurve and BndLib_Add2dCurve at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BndLib_Add3dCurve` and `BndLib_Add2dCurve`: add the box of a curve to a box, through
//! GeomBndLib_Curve and GeomBndLib_Curve2d.
use crate::geom_bnd_lib::{Curve, Curve2d};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};

/// `BndLib_Add3dCurve`: the box of an `Adaptor3d_Curve` added to a `Bnd_Box`.
pub struct Add3dCurve;

impl Add3dCurve {
    /// `Add(C, Tol, B)`: the adaptor's parameter range; `Add3dCurve::add(&adaptor, 0.0, &mut bounds)?`.
    pub fn add(
        curve: &dyn Adaptor3dCurve,
        tolerance: f64,
        bounds: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        Self::add_range(
            curve,
            curve.first_parameter()?,
            curve.last_parameter()?,
            tolerance,
            bounds,
        )
    }
    /// `Add(C, U1, U2, Tol, B)`: `GeomBndLib_Curve(C).Add(U1, U2, Tol, B)`;
    /// `Add3dCurve::add_range(&adaptor, 0.0, 1.0, 0.0, &mut bounds)?`.
    pub fn add_range(
        curve: &dyn Adaptor3dCurve,
        u1: f64,
        u2: f64,
        tolerance: f64,
        bounds: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        Curve::from_adaptor(curve)?.add_range(u1, u2, tolerance, bounds)
    }
    /// `AddOptimal(C, Tol, B)`; `Add3dCurve::add_optimal(&adaptor, 0.0, &mut bounds)?`.
    pub fn add_optimal(
        curve: &dyn Adaptor3dCurve,
        tolerance: f64,
        bounds: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        Self::add_optimal_range(
            curve,
            curve.first_parameter()?,
            curve.last_parameter()?,
            tolerance,
            bounds,
        )
    }
    /// `AddOptimal(C, U1, U2, Tol, B)`: `GeomBndLib_Curve(C).AddOptimal(U1, U2, Tol, B)`;
    /// `Add3dCurve::add_optimal_range(&adaptor, 0.0, 1.0, 0.0, &mut bounds)?`.
    pub fn add_optimal_range(
        curve: &dyn Adaptor3dCurve,
        u1: f64,
        u2: f64,
        tolerance: f64,
        bounds: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        Curve::from_adaptor(curve)?.add_optimal_range(u1, u2, tolerance, bounds)
    }
}

/// `BndLib_Add2dCurve`: the box of a 2D curve (adaptor or Geom2d handle) added to a
/// `Bnd_Box2d`.
pub struct Add2dCurve;

// `N = 33` points of the non-Geom2dAdaptor fallback.
const FALLBACK_POINTS: i32 = 33;

impl Add2dCurve {
    /// `Add(C, Tol, B)`: the adaptor's parameter range; `Add2dCurve::add_adaptor(&adaptor, 0.0, &mut bounds)?`.
    pub fn add_adaptor(
        curve: &dyn Adaptor2dCurve2d,
        tolerance: f64,
        bounds: &mut BoundingBox2,
    ) -> Result<(), Failure> {
        Self::add_adaptor_range(
            curve,
            curve.first_parameter()?,
            curve.last_parameter()?,
            tolerance,
            bounds,
        )
    }
    /// `Add(C, U1, U2, Tol, B)`: a Geom2dAdaptor_Curve goes through its Geom2d curve;
    /// any other adaptor adds 33 points `U1 + j (U2 - U1) / 32` (the parameter
    /// accumulated by addition, the last point at U2 exactly) and enlarges by `tol`;
    /// `Add2dCurve::add_adaptor_range(&adaptor, 0.0, 1.0, 0.0, &mut bounds)?`.
    pub fn add_adaptor_range(
        curve: &dyn Adaptor2dCurve2d,
        u1: f64,
        u2: f64,
        tolerance: f64,
        bounds: &mut BoundingBox2,
    ) -> Result<(), Failure> {
        if let Some(geom_adaptor) = Geom2dAdaptorCurve::project_from(curve) {
            return Self::add_curve_range(&geom_adaptor.curve(), u1, u2, tolerance, bounds);
        }
        let step: f64 = (u2 - u1) / f64::from(FALLBACK_POINTS - 1);
        let mut u: f64 = u1;
        for _ in 1..FALLBACK_POINTS {
            let point = curve.d0(u)?;
            u += step;
            bounds.add_point(point);
        }
        bounds.add_point(curve.d0(u2)?);
        bounds.enlarge(tolerance);
        Ok(())
    }
    /// `Add(occ::handle<Geom2d_Curve>, Tol, B)`: the curve's parameter range;
    /// `Add2dCurve::add_curve(&curve, 0.0, &mut bounds)?`.
    pub fn add_curve(
        curve: &Handle<dyn Geom2dCurve>,
        tolerance: f64,
        bounds: &mut BoundingBox2,
    ) -> Result<(), Failure> {
        let geometry = crate::geom_bnd_lib::live(curve, "BndLib_Add2dCurve")?;
        Self::add_curve_range(
            curve,
            geometry.first_parameter(),
            geometry.last_parameter(),
            tolerance,
            bounds,
        )
    }
    /// `Add(occ::handle<Geom2d_Curve>, T1, T2, Tol, B)`: `GeomBndLib_Curve2d(C).Add(...)`;
    /// `Add2dCurve::add_curve_range(&curve, 0.0, 1.0, 0.0, &mut bounds)?`.
    pub fn add_curve_range(
        curve: &Handle<dyn Geom2dCurve>,
        t1: f64,
        t2: f64,
        tolerance: f64,
        bounds: &mut BoundingBox2,
    ) -> Result<(), Failure> {
        Curve2d::from_curve(curve)?.add_range(t1, t2, tolerance, bounds)
    }
    /// `AddOptimal(occ::handle<Geom2d_Curve>, T1, T2, Tol, B)`;
    /// `Add2dCurve::add_optimal(&curve, 0.0, 1.0, 0.0, &mut bounds)?`.
    pub fn add_optimal(
        curve: &Handle<dyn Geom2dCurve>,
        t1: f64,
        t2: f64,
        tolerance: f64,
        bounds: &mut BoundingBox2,
    ) -> Result<(), Failure> {
        Curve2d::from_curve(curve)?.add_optimal_range(t1, t2, tolerance, bounds)
    }
}
