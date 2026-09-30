// Rust translation of FoundationClasses/TKMath/Convert at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Convert: exact B-spline representations of conics, elementary surfaces,
//! composite Bezier curves and piecewise polynomial curves and surfaces.
//!
//! The OCCT subclasses of `Convert_ConicToBSplineCurve` and
//! `Convert_ElementarySurfaceToBSplineSurface` add constructors only, so each
//! becomes a named constructor of [`ConicToBSplineCurve`] or
//! [`ElementarySurfaceToBSplineSurface`] (`from_circle`, `from_sphere`, ...).
//! C++ exceptions become [`Failure`] values of the same exception type; native
//! array accesses outside their bounds, which the reference build checks, become
//! Standard_OutOfRange failures. Where the native code writes past a fixed-size
//! stack buffer (undefined behaviour), the port fails with Standard_OutOfRange
//! before the write.
//!
//! [`Failure`]: rustycad_tkernel::standard::Failure
mod arrays;
mod circle;
mod comp_bezier;
mod comp_polynomial;
mod comp_polynomial_perform;
mod conic;
mod cos_sin;
mod cos_sin_eval;
mod cos_sin_periodic;
mod cylinder;
mod elementary;
mod grid_polynomial;
mod grid_polynomial_perform;
mod hyperbola;
mod polynomial_cos_sin;
mod revolution;
mod sphere;
mod torus;

pub use comp_bezier::{
    BezierJunctionPoint, CompBezierCurves2dToBSplineCurve2d, CompBezierCurvesToBSplineCurve,
    CompBezierCurvesToBSplineCurveBase,
};
pub use comp_polynomial::{CompPolynomialToPoles, PolynomialSpans};
pub use conic::ConicToBSplineCurve;
pub use cos_sin::CosAndSin;
pub use elementary::ElementarySurfaceToBSplineSurface;
pub use grid_polynomial::{GridPolynomialToPoles, PolynomialGrid};
pub use polynomial_cos_sin::build_polynomial_cos_and_sin;

use crate::gp::Point2;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// Parameterisation of a circle or ellipse arc (`Convert_ParameterisationType`);
/// `ParameterisationType::QuasiAngular as i32 == 5`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum ParameterisationType {
    /// Rational quadratic spans of at most 150 degrees each (the default).
    TgtThetaOver2,
    /// One rational quadratic span; arcs up to 0.9999 * PI.
    TgtThetaOver2_1,
    /// Two rational quadratic spans; arcs up to 1.9999 * PI.
    TgtThetaOver2_2,
    /// Three rational quadratic spans.
    TgtThetaOver2_3,
    /// Four rational quadratic spans.
    TgtThetaOver2_4,
    /// One rational span of degree 6 close to the angular parameterisation.
    QuasiAngular,
    /// Two rational spans of degree 4 joined with C1 continuity.
    RationalC1,
    /// One polynomial (non-rational) span of degree 7 approximating the arc.
    Polynomial,
}

/// The evaluator of `U(t)`, `V(t)` that AlgorithmicCosAndSin interpolates
/// (`Convert_CosAndSinEvalFunction`): parameter, evaluation degree, poles, knots
/// and optional multiplicities in; the two values out;
/// `let f: CosAndSinEvalFunction = my_evaluator;`.
pub type CosAndSinEvalFunction = fn(
    f64,
    i32,
    &Array1<'_, Point2>,
    &Array1<'_, f64>,
    Option<&Array1<'_, i32>>,
) -> Result<[f64; 2], Failure>;
