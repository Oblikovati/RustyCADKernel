//! `RefineBracketedRoot` of MathRoot_MultipleUtils.hxx: Brent, then bounded Newton polish.
use super::{MAX_NEWTON_POLISH_ITERATIONS, MultipleBrentValueWrapper, add_root};
use crate::math::FunctionWithDerivative;
use crate::math_root::{MultipleResult, brent};
use crate::math_utils::{Config, ScalarResult, Status};
use std::cell::RefCell;

/// Bracket, tolerances and budget of one refinement, bundled for the private helpers.
struct MultipleBracket {
    x1: f64,
    x2: f64,
    tolerance: f64,
    f_tolerance: f64,
    max_iterations: u32,
}

/// Refines a bracketed sign change of `f - offset` like math_FunctionRoots: Brent, then up
/// to [`MAX_NEWTON_POLISH_ITERATIONS`] Newton steps kept only while they stay in the bracket
/// and shrink the residual; adds the root and returns the status (`RefineBracketedRoot`).
/// `refine_bracketed_root(&mut f, 0.0, 0.0, -1.0, 2.0, 3.0, 1e-10, 1e-10, 100, 1e-10, &mut result)`.
#[allow(clippy::too_many_arguments)] // Keep the upstream eleven-argument public contract.
pub fn refine_bracketed_root<F: FunctionWithDerivative>(
    function: &mut F,
    offset: f64,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    tolerance: f64,
    f_tolerance: f64,
    max_iterations: u32,
    eps_x: f64,
    result: &mut MultipleResult,
) -> Status {
    if !y1.is_finite() || !y2.is_finite() {
        return Status::NumericalError;
    }
    let bracket = MultipleBracket {
        x1,
        x2,
        tolerance,
        f_tolerance,
        max_iterations,
    };
    let refined: ScalarResult = bracketed_brent(function, offset, &bracket);
    result.iterations = result.iterations.wrapping_add(refined.iterations);
    let root: f64 = match refined.root {
        Some(root) if refined.is_done() => root,
        _ => return refined.status,
    };
    let mut root_value: f64 = 0.0;
    if !function.value(root, &mut root_value) {
        return Status::CallbackError;
    }
    if !root_value.is_finite() {
        return Status::NumericalError;
    }
    match polish(function, offset, &bracket, (root, root_value)) {
        Ok((root, value)) => add_root(result, eps_x, root, value),
        Err(status) => return status,
    }
    Status::Ok
}

fn bracketed_brent<F: FunctionWithDerivative>(
    function: &mut F,
    offset: f64,
    bracket: &MultipleBracket,
) -> ScalarResult {
    let shared = RefCell::new(function);
    let mut wrapper = MultipleBrentValueWrapper::new(&shared, offset);
    let config: Config = Config {
        x_tolerance: bracket.tolerance,
        f_tolerance: bracket.f_tolerance,
        max_iterations: bracket.max_iterations,
        ..Default::default()
    };
    brent(&mut wrapper, bracket.x1, bracket.x2, &config)
}

fn polish<F: FunctionWithDerivative>(
    function: &mut F,
    offset: f64,
    bracket: &MultipleBracket,
    (mut root, mut root_value): (f64, f64),
) -> Result<(f64, f64), Status> {
    let mut shifted: f64 = root_value - offset;
    for _ in 0..MAX_NEWTON_POLISH_ITERATIONS {
        let Some(candidate) = newton_candidate(function, offset, bracket, root)? else {
            break;
        };
        let mut candidate_value: f64 = 0.0;
        if !function.value(candidate, &mut candidate_value) {
            return Err(Status::CallbackError);
        }
        if !candidate_value.is_finite() {
            return Err(Status::NumericalError);
        }
        let candidate_shifted: f64 = candidate_value - offset;
        if candidate_shifted.abs() >= shifted.abs() {
            break;
        }
        (root, root_value, shifted) = (candidate, candidate_value, candidate_shifted);
    }
    Ok((root, root_value))
}

fn newton_candidate<F: FunctionWithDerivative>(
    function: &mut F,
    offset: f64,
    bracket: &MultipleBracket,
    root: f64,
) -> Result<Option<f64>, Status> {
    let (mut value, mut derivative): (f64, f64) = (0.0, 0.0);
    if !function.values(root, &mut value, &mut derivative) {
        return Err(Status::CallbackError);
    }
    value -= offset;
    if !value.is_finite() || !derivative.is_finite() {
        return Err(Status::NumericalError);
    }
    if derivative == 0.0 {
        return Ok(None);
    }
    let candidate: f64 = root - value / derivative;
    // `candidate < x1 || candidate > x2` is false for NaN, hence the explicit finiteness test.
    if !candidate.is_finite() || candidate < bracket.x1 || candidate > bracket.x2 {
        return Ok(None);
    }
    Ok(Some(candidate))
}
