// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecCC.cxx (the fields and the static helpers Curvature and DiscretPar) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The state of `Bisector_BisecCC` and the small helpers of its algorithms.
use crate::bisector::PolyBis;
use crate::bisector::intervals::Intervals;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, errors};
use rustycad_tkmath::gp::Point2;

/// The fields of `Bisector_BisecCC`. A curve built by `new Bisector_BisecCC(...)`
/// starts from zeroed memory (the native allocator clears it), which the default
/// here reproduces; the default constructor then marks it empty.
#[derive(Clone, Default)]
pub(crate) struct CcState {
    pub curve1: Handle<dyn Geom2dCurve>,
    pub curve2: Handle<dyn Geom2dCurve>,
    pub sign1: f64,
    pub sign2: f64,
    pub start_intervals: Intervals,
    pub end_intervals: Intervals,
    pub current_interval: i32,
    pub my_polygon: PolyBis,
    pub shift_parameter: f64,
    pub dist_max: f64,
    pub is_empty: bool,
    pub is_convex1: bool,
    pub is_convex2: bool,
    pub extension_start: bool,
    pub extension_end: bool,
    pub point_start: Point2,
    pub point_end: Point2,
}

/// The in-out parameters of `ValueAndDist`, `ValueByInt` and `Extension`: the
/// parameters on the two curves and the square distance. A path that does not
/// write one keeps the caller's value (uninitialised natively, 0 in the port).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct OnCurves {
    pub u1: f64,
    pub u2: f64,
    pub dist: f64,
}

impl CcState {
    /// `LinkBisCurve(U)`: `U - shiftParameter`.
    pub(crate) fn link_bis_curve(&self, u: f64) -> f64 {
        u - self.shift_parameter
    }
    /// `LinkCurveBis(U)`: `U + shiftParameter`.
    pub(crate) fn link_curve_bis(&self, u: f64) -> f64 {
        u + self.shift_parameter
    }
    /// `FirstParameter()`: the start of the first interval.
    pub(crate) fn first_parameter(&self) -> Result<f64, Failure> {
        self.start_intervals.first()
    }
    /// `LastParameter()`: the end of the last interval.
    pub(crate) fn last_parameter(&self) -> Result<f64, Failure> {
        self.end_intervals.last()
    }
    /// `Curve(I)`: curve 1 or 2, Standard_OutOfRange otherwise.
    pub(crate) fn curve(&self, index: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        match index {
            1 => Ok(self.curve1.clone()),
            2 => Ok(self.curve2.clone()),
            _ => Err(errors::out_of_range(&format!(
                "Bisector_BisecCC::Curve: index {index}; expected 1 or 2"
            ))),
        }
    }
}

/// `Curvature(C, U, Tol)`: the signed curvature, 0 where the square norm of the
/// tangent is below `tol`.
pub(crate) fn curvature(c: &dyn Geom2dCurve, u: f64, tol: f64) -> Result<f64, Failure> {
    let (_, d1, d2) = c.d2(u)?;
    let norm2: f64 = d1.square_magnitude();
    if norm2 < tol {
        return Ok(0.0);
    }
    Ok(d1.crossed(d2) / (norm2 * norm2.sqrt()))
}

/// `DiscretPar(DU, EpsMin, EpsMax, NbMin, NbMax, Eps, Nb)`: the sampling step and
/// count of a parameter range; the flag is false when the range is too short.
pub(crate) fn discret_par(
    du: f64,
    eps_min: f64,
    eps_max: f64,
    nb_min: i32,
    nb_max: i32,
) -> (bool, f64, i32) {
    if du <= f64::from(nb_min) * eps_min {
        return (false, du / f64::from(nb_min + 1), nb_min);
    }
    let eps: f64 = rustycad_tkg2d::geom2d::native::std_min(eps_max, du / f64::from(nb_max));
    if eps < eps_min {
        // `int(DU / EpsMin)` truncates toward zero.
        return (true, eps_min, (du / eps_min) as i32);
    }
    (true, eps, nb_max)
}
