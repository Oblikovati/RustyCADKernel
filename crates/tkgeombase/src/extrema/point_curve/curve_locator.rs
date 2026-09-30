// Rust translation of Extrema_GCurveLocator.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GCurveLocator`: the nearest of a regular sampling of a curve.
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::failures::out_of_range;
use crate::extrema::gp_space::GpPoint;
use crate::extrema::pon::POnCurve;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use std::marker::PhantomData;

/// `Extrema_GCurveLocator` over one [`ExtremaCurveTool`]: two static members. The
/// native loop runs `NbU - 1` samples, so the last parameter is never evaluated;
/// `GCurveLocator::<CurveTool>::locate(point, &adaptor, 10)?`.
pub struct GCurveLocator<T: ExtremaCurveTool>(PhantomData<T>);

impl<T: ExtremaCurveTool> GCurveLocator<T> {
    /// `Locate(P, C, NbU, Papp)`: the nearest of `C(ui)`, `ui` from FirstParameter by
    /// `(Last - First) / (NbU - 1)`; Standard_OutOfRange when `NbU < 2`;
    /// `GCurveLocator::<CurveTool>::locate(point, &adaptor, 10)?`.
    pub fn locate(
        point: T::Point,
        curve: &T::Curve,
        nb_u: i32,
    ) -> Result<POnCurve<T::Point>, Failure> {
        check_samples(nb_u)?;
        let first: f64 = T::first_parameter(curve)?;
        let step: f64 = (T::last_parameter(curve)? - first) / f64::from(nb_u - 1);
        nearest_sample::<T>(point, curve, nb_u, first, step)
    }
    /// `Locate(P, C, NbU, Umin, Usup, Papp)`: the same between `Umin` and `Usup`,
    /// clipped to the curve range within RealEpsilon();
    /// `GCurveLocator::<CurveTool>::locate_range(point, &adaptor, 10, 0.0, 1.0)?`.
    pub fn locate_range(
        point: T::Point,
        curve: &T::Curve,
        nb_u: i32,
        umin: f64,
        usup: f64,
    ) -> Result<POnCurve<T::Point>, Failure> {
        check_samples(nb_u)?;
        let (uinf, ulast) = (T::first_parameter(curve)?, T::last_parameter(curve)?);
        let (u1, u2) = (std_min(uinf, ulast), std_max(uinf, ulast));
        let (mut u11, mut u12) = (std_min(umin, usup), std_max(umin, usup));
        if u11 < u1 - f64::EPSILON {
            u11 = u1;
        }
        if u12 > u2 + f64::EPSILON {
            u12 = u2;
        }
        let step: f64 = (u12 - u11) / f64::from(nb_u - 1);
        nearest_sample::<T>(point, curve, nb_u, u11, step)
    }
}

fn check_samples(nb_u: i32) -> Result<(), Failure> {
    if nb_u < 2 {
        return Err(out_of_range("Extrema_GCurveLocator::Locate()", nb_u, 2));
    }
    Ok(())
}

// The nearest of the `NbU - 1` samples from `start` by `step` (accumulated, as the
// native `aU += aPasU`).
fn nearest_sample<T: ExtremaCurveTool>(
    point: T::Point,
    curve: &T::Curve,
    nb_u: i32,
    start: f64,
    step: f64,
) -> Result<POnCurve<T::Point>, Failure> {
    let (mut u, mut dist2_min, mut u_min) = (start, f64::MAX, 0.0);
    let mut p_min: T::Point = T::Point::default();
    for _ in 1..nb_u {
        let pt: T::Point = T::value(curve, u)?;
        let dist2: f64 = pt.square_distance_to(&point);
        if dist2 < dist2_min {
            dist2_min = dist2;
            u_min = u;
            p_min = pt;
        }
        u += step;
    }
    Ok(POnCurve::new(u_min, p_min))
}
