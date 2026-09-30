// Rust translation of FoundationClasses/TKMath/PLib/PLib.cxx (EvalLagrange, EvalCubicHermite) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Divided-difference interpolation evaluated in Newton form.
use super::access::{non_negative, require_length};
use rustycad_tkernel::standard::Failure;

const LAGRANGE: &str = "PLib::EvalLagrange";
const HERMITE: &str = "PLib::EvalCubicHermite";
const CUBIC: usize = 3;

/// Evaluates the Lagrange interpolant of Degree+1 points and its derivatives
/// (`PLib::EvalLagrange`).
///
/// Returns 1 without touching `results` when two parameters are closer than
/// `RealSmall()`, otherwise 0. A negative request evaluates only the value, as
/// natively; `plib::eval_lagrange(0.5, 1, 1, 1, &[0.0, 1.0], &[0.0, 1.0], &mut out)?`.
pub fn eval_lagrange(
    parameter: f64,
    derivative_request: i32,
    degree: i32,
    dimension: i32,
    values: &[f64],
    parameters: &[f64],
    results: &mut [f64],
) -> Result<i32, Failure> {
    let order: usize = non_negative(LAGRANGE, "Degree", degree)?;
    let dim: usize = non_negative(LAGRANGE, "Dimension", dimension)?;
    let request: i32 = derivative_request.min(degree);
    require_length(LAGRANGE, "Values", values.len(), (order + 1) * dim)?;
    require_length(LAGRANGE, "Parameters", parameters.len(), order + 1)?;
    require_length(
        LAGRANGE,
        "Results",
        results.len(),
        result_length(request, dim),
    )?;
    let mut divided: Vec<f64> = values[..(order + 1) * dim].to_vec();
    if !lagrange_differences(order, dim, parameters, &mut divided) {
        return Ok(1);
    }
    newton_form(
        parameter, request, order, dim, &divided, parameters, results,
    );
    Ok(0)
}

// The value row is always written, so a negative request still needs Dimension slots.
fn result_length(request: i32, dim: usize) -> usize {
    (request.max(0) as usize + 1) * dim
}

fn lagrange_differences(order: usize, dim: usize, parameters: &[f64], divided: &mut [f64]) -> bool {
    for ii in (0..=order).rev() {
        for jj in (order - ii + 1..=order).rev() {
            let index: usize = jj * dim;
            for kk in 0..dim {
                divided[index + kk] -= divided[index - dim + kk];
            }
            let difference: f64 = parameters[jj] - parameters[jj + ii - order - 1];
            if difference.abs() < f64::MIN_POSITIVE {
                return false;
            }
            let inverse: f64 = 1.0e0 / difference;
            for kk in 0..dim {
                divided[index + kk] *= inverse;
            }
        }
    }
    true
}

/// Evaluates the cubic Hermite interpolant of two end values and end derivatives
/// (`PLib::EvalCubicHermite`); always returns 0.
///
/// Equal end parameters divide by zero exactly as natively;
/// `plib::eval_cubic_hermite(0.5, 1, 1, &[0.0, 1.0], &[1.0, 0.0], &[0.0, 1.0], &mut out)?`.
pub fn eval_cubic_hermite(
    parameter: f64,
    derivative_request: i32,
    dimension: i32,
    values: &[f64],
    derivatives: &[f64],
    parameters: &[f64],
    results: &mut [f64],
) -> Result<i32, Failure> {
    let dim: usize = non_negative(HERMITE, "Dimension", dimension)?;
    let request: i32 = derivative_request.min(CUBIC as i32);
    require_length(HERMITE, "Values", values.len(), 2 * dim)?;
    require_length(HERMITE, "Derivatives", derivatives.len(), 2 * dim)?;
    require_length(HERMITE, "Parameters", parameters.len(), 2)?;
    require_length(
        HERMITE,
        "Results",
        results.len(),
        result_length(request, dim),
    )?;
    let nodes: [f64; 4] = [parameters[0], parameters[0], parameters[1], parameters[1]];
    let inverse: f64 = 1.0e0 / (parameters[1] - parameters[0]);
    let mut divided: Vec<f64> = vec![0.0; (CUBIC + 1) * dim];
    for ii in 0..dim {
        divided[ii] = values[ii];
        divided[2 * dim + ii] = inverse * (values[dim + ii] - values[ii]);
        divided[dim + ii] = derivatives[ii];
        divided[3 * dim + ii] = derivatives[dim + ii];
    }
    hermite_differences(dim, inverse, &mut divided);
    newton_form(parameter, request, CUBIC, dim, &divided, &nodes, results);
    Ok(0)
}

// Stage 2 of the divided-difference table is seeded above; finish stages 2 and 3.
fn hermite_differences(dim: usize, inverse: f64, divided: &mut [f64]) {
    for ii in 1..=CUBIC {
        for jj in (ii + 1..=CUBIC).rev() {
            let index: usize = jj * dim;
            for kk in 0..dim {
                divided[index + kk] -= divided[index - dim + kk];
            }
            for kk in 0..dim {
                divided[index + kk] *= inverse;
            }
        }
    }
}

// P(t) = [t1]P + (t-t1)[t1,t2]P + ... evaluated with derivative rows by synthetic division.
fn newton_form(
    parameter: f64,
    request: i32,
    order: usize,
    dim: usize,
    divided: &[f64],
    nodes: &[f64],
    results: &mut [f64],
) {
    let request: usize = request.max(0) as usize;
    results[..dim].copy_from_slice(&divided[order * dim..(order + 1) * dim]);
    results[dim..(request + 1) * dim].fill(0.0e0);
    for ii in (1..=order).rev() {
        let difference: f64 = parameter - nodes[ii - 1];
        for jj in (1..=request).rev() {
            for kk in 0..dim {
                results[jj * dim + kk] *= difference;
                results[jj * dim + kk] += results[(jj - 1) * dim + kk] * jj as f64;
            }
        }
        for kk in 0..dim {
            results[kk] *= difference;
            results[kk] += divided[(ii - 1) * dim + kk];
        }
    }
}
