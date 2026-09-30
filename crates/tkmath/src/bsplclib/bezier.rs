// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_BzSyntaxes.cxx (D0..D3) and BSplCLib_CurveComputation.pxx (BSplCLib_KnotArrays) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Bezier curves evaluated as B-splines with knots {0, 1} of multiplicity Degree+1.
//!
//! `BSplCLib::DN(U, N, Poles, Weights, P, VN)` is declared for Bezier poles in the
//! pinned header but has no definition in the pinned sources; it is not ported.
use super::curve_eval::{CurveSpan, d0, d1, d2, d3};
use super::point::{CurveD3, CurvePoint};
use super::views::one_based;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// Knots `{0, 1}` with multiplicities `{Degree+1, Degree+1}` (`BSplCLib_KnotArrays<2>`);
/// `bsplclib::bezier_knots(3)?.1[1] == 4`.
pub fn bezier_knots(degree: i32) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let mut knots: Array1<'static, f64> = one_based("BSplCLib_KnotArrays", 2)?;
    knots.as_mut_slice().copy_from_slice(&[0.0, 1.0]);
    let mut mults: Array1<'static, i32> = one_based("BSplCLib_KnotArrays", 2)?;
    mults.init(degree + 1);
    Ok((knots, mults))
}

fn bezier_span<'r, 'a>(
    u: f64,
    degree: i32,
    knots: &'r Array1<'a, f64>,
    mults: &'r Array1<'a, i32>,
) -> CurveSpan<'r, 'a> {
    CurveSpan {
        u,
        index: 1,
        degree,
        periodic: false,
        knots,
        mults: Some(mults),
    }
}

/// Point of a Bezier curve (`BSplCLib::D0(U, Poles, Weights, P)`);
/// `bsplclib::bezier_d0(0.5, &poles, None)?`.
pub fn bezier_d0<P: CurvePoint>(
    u: f64,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<P, Failure> {
    let degree: i32 = poles.len() as i32 - 1;
    let (knots, mults) = bezier_knots(degree)?;
    d0(&bezier_span(u, degree, &knots, &mults), poles, weights)
}

/// Point and derivative of a Bezier curve (`BSplCLib::D1(U, Poles, Weights, P, V)`);
/// `bsplclib::bezier_d1(0.5, &poles, None)?`.
pub fn bezier_d1<P: CurvePoint>(
    u: f64,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(P, P::Vector), Failure> {
    let degree: i32 = poles.len() as i32 - 1;
    let (knots, mults) = bezier_knots(degree)?;
    d1(&bezier_span(u, degree, &knots, &mults), poles, weights)
}

/// Point and two derivatives of a Bezier curve (`BSplCLib::D2(U, Poles, Weights, ...)`);
/// `bsplclib::bezier_d2(0.5, &poles, None)?`.
pub fn bezier_d2<P: CurvePoint>(
    u: f64,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<(P, P::Vector, P::Vector), Failure> {
    let degree: i32 = poles.len() as i32 - 1;
    let (knots, mults) = bezier_knots(degree)?;
    d2(&bezier_span(u, degree, &knots, &mults), poles, weights)
}

/// Point and three derivatives of a Bezier curve (`BSplCLib::D3(U, Poles, Weights, ...)`);
/// `bsplclib::bezier_d3(0.5, &poles, None)?`.
pub fn bezier_d3<P: CurvePoint>(
    u: f64,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<CurveD3<P>, Failure> {
    let degree: i32 = poles.len() as i32 - 1;
    let (knots, mults) = bezier_knots(degree)?;
    d3(&bezier_span(u, degree, &knots, &mults), poles, weights)
}
