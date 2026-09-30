// Rust translation of ComputeBoundsfromInfinite of
// ModelingAlgorithms/TKGeomAlgo/IntStart/IntStart_SearchOnBoundaries.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The coarse bounds of an infinite arc ("PROVISIONAL - TEMPORARY - NOT GOOD - NYI"
//! natively): Newton steps of the signed distance from three far points.
use super::value_or_keep;
use crate::int_start::tools::TheArcFunction;
use rustycad_tkernel::standard::Failure;

/// The step of the difference quotients.
const D_U: f64 = 0.001;

/// `ComputeBoundsfromInfinite(Func, PDeb, PFin, NbEchant)`: `(PDeb, PFin)`. The native
/// `NbEchant` output (always 100) is not read by its caller. `(1e10, -1e10)` marks an
/// arc that may lie on the quadric.
pub(super) fn compute_bounds_from_infinite<F: TheArcFunction>(
    func: &mut F,
) -> Result<(f64, f64), Failure> {
    // The native Dist0 and Dist1 are uninitialized; see `value_or_keep`.
    let mut dists: [f64; 2] = [0.0; 2];
    let mut u0: f64 = 0.0;
    let d_dist: f64 = quotient(func, u0, &mut dists)?;
    // `if (dDist)`: NaN is true natively, as `!= 0.0` is here.
    if d_dist == 0.0 {
        return Ok((1e10, -1e10));
    }
    u0 -= D_U * dists[0] / d_dist;
    let mut umin: f64 = newton_bound(func, u0 - 1e5, -10.0, &mut dists)?;
    let mut umax: f64 = newton_bound(func, u0 + 1e8, 10.0, &mut dists)?;
    if umin > u0 {
        umin = u0 - 10.0;
    }
    if umax < u0 {
        umax = u0 + 10.0;
    }
    let pfin: f64 = umax + 10. * (umax - umin);
    let pdeb: f64 = umin - 10. * (umax - umin);
    Ok((pdeb, pfin))
}

/// The distances at `u` and `u + dU` (kept in `dists`) and their difference.
fn quotient<F: TheArcFunction>(func: &mut F, u: f64, dists: &mut [f64; 2]) -> Result<f64, Failure> {
    dists[0] = value_or_keep(func, u, dists[0])?;
    dists[1] = value_or_keep(func, u + D_U, dists[1])?;
    Ok(dists[1] - dists[0])
}

/// One Newton step from `u`, or `u + shift` when the distance is flat there.
fn newton_bound<F: TheArcFunction>(
    func: &mut F,
    mut u: f64,
    shift: f64,
    dists: &mut [f64; 2],
) -> Result<f64, Failure> {
    let d_dist: f64 = quotient(func, u, dists)?;
    if d_dist != 0.0 {
        u -= D_U * dists[0] / d_dist;
    } else {
        u += shift;
    }
    Ok(u)
}
