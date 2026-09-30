// Rust translation of ModelingData/TKG3d/AdvApprox/AdvApprox_ApproxAFunction.cxx (Approximation) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApprox_ApproxAFunction::Approximation`: least-squares approximation of one
//! function by up to `MaxSegments` polynomial pieces, splitting an interval while
//! its error exceeds a subspace tolerance.
use super::cutting::AdvApproxCutting;
use super::evaluator::AdvApproxEvaluatorFunction;
use super::simple_approx::AdvApproxSimpleApprox;
use crate::geom::errors;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::plib::{self, JacobiPolynomial};

/// The inputs of `Approximation`, in native order.
pub struct AdvApproxApproximationInput<'r, 'a> {
    /// Total space dimension (sum of the subspace dimensions).
    pub total_dimension: i32,
    /// Number of independent subspaces.
    pub total_num_ss: i32,
    /// Dimension of each subspace, read at `1..=TotalNumSS`.
    pub local_dimension: &'r Array1<'a, i32>,
    /// First parameter of the function.
    pub first: f64,
    /// Last parameter of the function.
    pub last: f64,
    /// Continuity order 0, 1 or 2 at the ends of each piece.
    pub continuity_order: i32,
    /// Limit number of coefficients of a piece (>= 2 * ContinuityOrder + 2).
    pub num_max_coeffs: i32,
    /// Maximum number of pieces.
    pub max_segments: i32,
    /// Tolerance of each subspace, read at `1..=TotalNumSS`.
    pub tolerances: &'r Array1<'a, f64>,
    /// Discretisation code of `PLib::JacobiParameters` (1 to 4).
    pub code_precis: i32,
}

/// The outputs of `Approximation`: caller-sized tables, read and written at native
/// absolute indices, the piece count and the error code.
pub struct AdvApproxApproximationOutput<'r, 'a> {
    /// Number of pieces created (`NumCurves`).
    pub num_curves: i32,
    /// Coefficient count of each piece, at `1..=NumCurves`.
    pub num_coeff_per_curve: &'r mut Array1<'a, i32>,
    /// Canonical coefficients, `NumMaxCoeffs * TotalDimension` per piece from index 1.
    pub coefficients: &'r mut Array1<'a, f64>,
    /// The `NumCurves + 1` breakpoints, from index 1.
    pub intervals: &'r mut Array1<'a, f64>,
    /// Maximum error of each piece and subspace, `TotalNumSS` per piece from index 1.
    pub error_max: &'r mut Array1<'a, f64>,
    /// Average error of each piece and subspace.
    pub average_error: &'r mut Array1<'a, f64>,
    /// 0 when everything is fine, 1 for inconsistent input or an evaluation error.
    pub error_code: i32,
}

// The continuity of a constraint order; Standard_ConstructionError beyond 2.
fn continuity_of(order: i32) -> Result<Shape, Failure> {
    match order {
        0 => Ok(Shape::C0),
        1 => Ok(Shape::C1),
        2 => Ok(Shape::C2),
        other => Err(errors::construction(&format!(
            "AdvApprox_ApproxAFunction::Approximation: ContinuityOrder = {other}; expected 0, 1 or 2"
        ))),
    }
}

// The native TABINT, a 0-based view of IntervalsArray(1..).
fn tab(intervals: &Array1<'_, f64>, k: i32) -> Result<f64, Failure> {
    at(intervals, k + 1)
}

/// `Approximation`: fills `output`; `error_code` 1 reports inconsistent input or
/// an evaluation error, as natively. Invalid continuity orders and Jacobi
/// parameters raise Standard_ConstructionError;
/// `approximation(&input, &mut f, &AdvApproxDichoCutting, &mut output)?`.
pub fn approximation(
    input: &AdvApproxApproximationInput<'_, '_>,
    evaluator: &mut dyn AdvApproxEvaluatorFunction,
    cut_tool: &dyn AdvApproxCutting,
    output: &mut AdvApproxApproximationOutput<'_, '_>,
) -> Result<(), Failure> {
    tab(output.intervals, 0)?;
    output.error_code = 0;
    output.coefficients.init(0.);
    if !consistent(input)? {
        output.error_code = 1;
        return Ok(());
    }
    let continuity: Shape = continuity_of(input.continuity_order)?;
    let (mut nb_gauss_points, mut work_degree) = (0, 0);
    plib::jacobi_parameters(
        continuity,
        input.num_max_coeffs - 1,
        input.code_precis,
        &mut nb_gauss_points,
        &mut work_degree,
    )?;
    put(output.intervals, 1, input.first)?;
    put(output.intervals, 2, input.last)?;
    output.num_curves = 0;
    let jacobi_base = JacobiPolynomial::new(work_degree, continuity)?;
    let mut approx = AdvApproxSimpleApprox::new(
        input.total_dimension,
        input.total_num_ss,
        continuity,
        work_degree,
        nb_gauss_points,
        &jacobi_base,
        evaluator,
    )?;
    split_loop(input, cut_tool, &jacobi_base, &mut approx, output)
}

// The native input checks that set ErrorCode = 1.
fn consistent(input: &AdvApproxApproximationInput<'_, '_>) -> Result<bool, Failure> {
    if input.max_segments < 1 || (input.last - input.first).abs() < 1.0e-9 {
        return Ok(false);
    }
    let mut idim: i32 = 0;
    for i in 1..=input.total_num_ss {
        idim += at(input.local_dimension, i)?;
    }
    Ok(idim == input.total_dimension)
}

// The piece loop: approximate the next stacked interval, split it while the error
// is too large and the stack allows it, else accept it.
fn split_loop(
    input: &AdvApproxApproximationInput<'_, '_>,
    cut_tool: &dyn AdvApproxCutting,
    jacobi_base: &JacobiPolynomial,
    approx: &mut AdvApproxSimpleApprox<'_>,
    output: &mut AdvApproxApproximationOutput<'_, '_>,
) -> Result<(), Failure> {
    let (mut nupil, mut is_cut) = (1, false);
    while nupil - output.num_curves != 0 {
        let nc: i32 = output.num_curves;
        let (a, b) = (tab(output.intervals, nc)?, tab(output.intervals, nc + 1)?);
        approx.perform(
            input.local_dimension,
            input.tolerances,
            a,
            b,
            input.num_max_coeffs - 1,
        )?;
        if !approx.is_done() {
            output.error_code = 1;
            return Ok(());
        }
        put(output.num_coeff_per_curve, nc + 1, 0)?;
        if !within_tolerances(input, approx)? {
            let (large, tmil) = cut_tool.value(a, b);
            if nupil < input.max_segments && large {
                is_cut = true;
                push_cut(output.intervals, nc, nupil, tmil)?;
                nupil += 1;
                continue;
            }
        }
        output.num_curves += 1;
        store_piece(input, jacobi_base, approx, is_cut, output)?;
    }
    Ok(())
}

// The error must be satisfied on every subspace, otherwise the interval is split.
fn within_tolerances(
    input: &AdvApproxApproximationInput<'_, '_>,
    approx: &AdvApproxSimpleApprox<'_>,
) -> Result<bool, Failure> {
    for is in 1..=input.total_num_ss {
        if approx.max_error(is)? > at(input.tolerances, is)? {
            return Ok(false);
        }
    }
    Ok(true)
}

// Shifts the stacked breakpoints after the current interval and inserts `tmil`.
fn push_cut(
    intervals: &mut Array1<'_, f64>,
    nc: i32,
    nupil: i32,
    tmil: f64,
) -> Result<(), Failure> {
    let ilong: i32 = nupil - nc - 1;
    for i in (0..=ilong).rev() {
        let value: f64 = tab(intervals, nc + 1 + i)?;
        put(intervals, nc + 2 + i + 1, value)?;
    }
    put(intervals, nc + 2, tmil)
}

// Errors, degree and canonical coefficients of the accepted piece `NumCurves`.
fn store_piece(
    input: &AdvApproxApproximationInput<'_, '_>,
    jacobi_base: &JacobiPolynomial,
    approx: &AdvApproxSimpleApprox<'_>,
    is_cut: bool,
    output: &mut AdvApproxApproximationOutput<'_, '_>,
) -> Result<(), Failure> {
    let (nc, td) = (output.num_curves, input.total_dimension);
    for is in 1..=input.total_num_ss {
        let index: i32 = is + (nc - 1) * input.total_num_ss;
        put(output.error_max, index, approx.max_error(is)?)?;
        put(output.average_error, index, approx.average_error(is)?)?;
    }
    let mut the_deg: i32 = approx.degree();
    if is_cut && the_deg < 2 * input.continuity_order + 1 {
        // Avoids noisy derivatives at the ends and keeps the continuity of the B-spline.
        the_deg = 2 * input.continuity_order + 1;
    }
    put(output.num_coeff_per_curve, nc, the_deg + 1)?;
    let mut coefficients: Array1<'static, f64> =
        Array1::with_bounds(0, (the_deg + 1) * td - 1).map_err(errors::array_bounds)?;
    jacobi_base.to_coefficients(td, the_deg, approx.coefficients(), &mut coefficients)?;
    let first: i32 = (nc - 1) * td * input.num_max_coeffs + 1;
    for (j, &value) in (first..).zip(coefficients.as_slice()) {
        put(output.coefficients, j, value)?;
    }
    Ok(())
}
