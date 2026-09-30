// Rust translation of the singular-derivative blocks that Extrema_GFuncExtPC.hxx and
// Extrema_GFuncExtCC.hxx repeat (for each curve) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The replacement of a vanishing first derivative in the extrema functions: the
//! first higher derivative above the tolerance, oriented along the curve, or a
//! three-point difference when every derivative up to the maximal order vanishes.
use super::curve_tool::ExtremaCurveTool;
use super::gp_space::{GpPoint, GpVector};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};

/// `TolFactor` / `THE_TOL_FACTOR`: the tolerance relative to the largest derivative.
pub(crate) const TOL_FACTOR: f64 = 1.0e-12;
/// `MinTol` / `THE_MIN_TOL`.
pub(crate) const MIN_TOL: f64 = 1.0e-20;
/// `MinStep` / `THE_MIN_STEP`.
pub(crate) const MIN_STEP: f64 = 1.0e-7;
/// `MaxOrder` / `THE_MAX_ORDER`.
pub(crate) const MAX_ORDER: i32 = 3;

/// The interval, tolerance and maximal derivative order of one curve.
pub(crate) struct SingularSearch {
    pub uinf: f64,
    pub usup: f64,
    pub tol: f64,
    pub max_order: i32,
}

/// `max(du * factor, MinStep)` with `du` null for an infinite interval
/// (`myUsupremum >= RealLast() || myUinfium <= RealFirst()`).
pub(crate) fn singular_step(uinf: f64, usup: f64, factor: f64) -> f64 {
    let du: f64 = if (usup >= f64::MAX) || (uinf <= f64::MIN) {
        0.0
    } else {
        usup - uinf
    };
    std_max(du * factor, MIN_STEP)
}

/// The derivative at `u` of the singular case (division factor 1e-3).
pub(crate) fn singular_derivative<T: ExtremaCurveTool>(
    curve: &T::Curve,
    u: f64,
    search: &SingularSearch,
) -> Result<T::Vector, Failure> {
    let delta: f64 = singular_step(search.uinf, search.usup, 1.0e-3);
    let mut n: i32 = 1;
    let (mut v, mut found): (T::Vector, bool);
    loop {
        n += 1;
        v = T::dn(curve, u, n)?;
        found = v.norm() > search.tol;
        if found || n >= search.max_order {
            break;
        }
    }
    if !found {
        return three_point_derivative::<T>(curve, u, search.uinf, delta);
    }
    let other: f64 = if u - search.uinf < delta {
        u + delta
    } else {
        u - delta
    };
    let p1: T::Point = T::d0(curve, std_min(u, other))?;
    let p2: T::Point = T::d0(curve, std_max(u, other))?;
    let dir_factor: f64 = v.dot_with(&p1.vector_to(&p2));
    Ok(if dir_factor < 0.0 { -v } else { v })
}

/// The derivative approximated by three points `delta` apart, forward near the
/// start of the interval and backward elsewhere, from vectors to the origin
/// (`TheVector V1(Ptemp, P1)` with a default `Ptemp`).
pub(crate) fn three_point_derivative<T: ExtremaCurveTool>(
    curve: &T::Curve,
    u: f64,
    uinf: f64,
    delta: f64,
) -> Result<T::Vector, Failure> {
    let origin: T::Point = T::Point::default();
    if u - uinf < 2.0 * delta {
        let v1 = origin.vector_to(&T::d0(curve, u)?);
        let v2 = origin.vector_to(&T::d0(curve, u + delta)?);
        let v3 = origin.vector_to(&T::d0(curve, u + 2.0 * delta)?);
        return Ok(v1 * -3.0 + v2 * 4.0 - v3);
    }
    let v1 = origin.vector_to(&T::d0(curve, u - 2.0 * delta)?);
    let v2 = origin.vector_to(&T::d0(curve, u - delta)?);
    let v3 = origin.vector_to(&T::d0(curve, u)?);
    Ok(v1 - v2 * 4.0 + v3 * 3.0)
}
