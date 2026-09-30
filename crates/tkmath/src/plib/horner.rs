// Rust translation of FoundationClasses/TKMath/PLib/PLib.cxx (EvalPolynomial family) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Horner evaluation of vector-valued polynomials and their derivatives.
//!
//! OCCT dispatches dimensions 1..15 to templated register-friendly copies and
//! larger dimensions to runtime loops. Both perform identical per-component
//! operations, so a single loop per derivative order reproduces them exactly.
//! The order-0 request deliberately takes the general branch, which starts from
//! `0 * U + c` rather than copying `c` (they differ for infinite or NaN `U`).
use super::access::{non_negative, require_length};
use rustycad_tkernel::standard::Failure;

const EVAL: &str = "PLib::EvalPolynomial";
const NO_DERIVATIVE: &str = "PLib::NoDerivativeEvalPolynomial";

/// Evaluates c0 + c1 U + ... + cDegree U^Degree and derivatives up to the request
/// (`PLib::EvalPolynomial`).
///
/// Coefficients are stored degree by degree, Dimension values each; results hold
/// (DerivativeRequest+1)*Dimension values;
/// `plib::eval_polynomial(2.0, 1, 2, 1, &[1.0, 2.0, 3.0], &mut out)?`.
pub fn eval_polynomial(
    u: f64,
    derivative_request: i32,
    degree: i32,
    dimension: i32,
    polynomial_coeff: &[f64],
    results: &mut [f64],
) -> Result<(), Failure> {
    let request: usize = non_negative(EVAL, "DerivativeRequest", derivative_request)?;
    let degree: usize = non_negative(EVAL, "Degree", degree)?;
    let dim: usize = non_negative(EVAL, "Dimension", dimension)?;
    require_length(
        EVAL,
        "PolynomialCoeff",
        polynomial_coeff.len(),
        (degree + 1) * dim,
    )?;
    require_length(EVAL, "Results", results.len(), (request + 1) * dim)?;
    let coefficients: &[f64] = &polynomial_coeff[..(degree + 1) * dim];
    match request {
        1 => value_and_first(u, degree, dim, coefficients, results),
        2 => value_to_second(u, degree, dim, coefficients, results),
        _ => general_derivatives(u, request, degree, dim, coefficients, results),
    }
    Ok(())
}

/// Evaluates only the value, reading the top coefficient at `DegreeDimension`
/// (`PLib::NoDerivativeEvalPolynomial`);
/// `plib::no_derivative_eval_polynomial(2.0, 2, 1, 2, &[1.0, 2.0, 3.0], &mut out)?`.
pub fn no_derivative_eval_polynomial(
    u: f64,
    degree: i32,
    dimension: i32,
    degree_dimension: i32,
    polynomial_coeff: &[f64],
    results: &mut [f64],
) -> Result<(), Failure> {
    let degree: usize = non_negative(NO_DERIVATIVE, "Degree", degree)?;
    let dim: usize = non_negative(NO_DERIVATIVE, "Dimension", dimension)?;
    let top: usize = non_negative(NO_DERIVATIVE, "DegreeDimension", degree_dimension)?;
    if dim == 0 {
        return Ok(());
    }
    // The native pointer walks back Degree rows of Dimension values from `top`.
    if top < degree * dim {
        return Err(Failure::out_of_range(&format!(
            "{NO_DERIVATIVE}: DegreeDimension = {top}; expected >= Degree*Dimension = {}",
            degree * dim
        )));
    }
    require_length(
        NO_DERIVATIVE,
        "PolynomialCoeff",
        polynomial_coeff.len(),
        top + dim,
    )?;
    require_length(NO_DERIVATIVE, "Results", results.len(), dim)?;
    let coefficients: &[f64] = &polynomial_coeff[top - degree * dim..top + dim];
    value_only(u, degree, dim, coefficients, results);
    Ok(())
}

fn value_only(u: f64, degree: usize, dim: usize, coefficients: &[f64], results: &mut [f64]) {
    results[..dim].copy_from_slice(&coefficients[degree * dim..]);
    for row in (0..degree).rev() {
        for i in 0..dim {
            results[i] = results[i] * u + coefficients[row * dim + i];
        }
    }
}

fn value_and_first(u: f64, degree: usize, dim: usize, coefficients: &[f64], results: &mut [f64]) {
    let (value, first) = results.split_at_mut(dim);
    value.copy_from_slice(&coefficients[degree * dim..]);
    first[..dim].fill(0.0);
    for row in (0..degree).rev() {
        for i in 0..dim {
            let previous: f64 = value[i];
            first[i] = first[i] * u + previous;
            value[i] = previous * u + coefficients[row * dim + i];
        }
    }
}

fn value_to_second(u: f64, degree: usize, dim: usize, coefficients: &[f64], results: &mut [f64]) {
    let (value, rest) = results.split_at_mut(dim);
    let (first, second) = rest.split_at_mut(dim);
    value.copy_from_slice(&coefficients[degree * dim..]);
    first.fill(0.0);
    second[..dim].fill(0.0);
    for row in (0..degree).rev() {
        for i in 0..dim {
            let slope: f64 = first[i];
            let previous: f64 = value[i];
            second[i] = second[i] * u + slope * 2.0;
            first[i] = slope * u + previous;
            value[i] = previous * u + coefficients[row * dim + i];
        }
    }
}

fn general_derivatives(
    u: f64,
    request: usize,
    degree: usize,
    dim: usize,
    coefficients: &[f64],
    results: &mut [f64],
) {
    results[..(request + 1) * dim].fill(0.0);
    for row in (0..=degree).rev() {
        for order in (1..=request).rev() {
            for i in 0..dim {
                results[order * dim + i] =
                    results[order * dim + i] * u + results[(order - 1) * dim + i] * order as f64;
            }
        }
        for i in 0..dim {
            results[i] = results[i] * u + coefficients[row * dim + i];
        }
    }
}

/// Evaluates a bivariate polynomial by two EvalPolynomial passes (`PLib::EvalPoly2Var`).
///
/// Coefficients are ordered U fastest, then V; the result is the requested mixed
/// derivative, Dimension values;
/// `plib::eval_poly_2var(0.5, 0.25, 1, 0, 1, 1, 1, &coeffs, &mut out)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native nine-argument signature.
pub fn eval_poly_2var(
    u: f64,
    v: f64,
    u_derivative_request: i32,
    v_derivative_request: i32,
    u_degree: i32,
    v_degree: i32,
    dimension: i32,
    polynomial_coeff: &[f64],
    results: &mut [f64],
) -> Result<(), Failure> {
    let name: &str = "PLib::EvalPoly2Var";
    let u_request: usize = non_negative(name, "UDerivativeRequest", u_derivative_request)?;
    let v_request: usize = non_negative(name, "VDerivativeRequest", v_derivative_request)?;
    let dim: usize = non_negative(name, "Dimension", dimension)?;
    let u_dim: usize = (non_negative(name, "VDegree", v_degree)? + 1) * dim;
    require_length(name, "Results", results.len(), dim)?;
    let mut curve: Vec<f64> = vec![0.0; u_dim * (u_request + 1)];
    let mut point: Vec<f64> = vec![0.0; dim * (v_request + 1)];
    let u_dim_i32: i32 = i32::try_from(u_dim).map_err(|_| too_wide(name, u_dim))?;
    eval_polynomial(
        u,
        u_derivative_request,
        u_degree,
        u_dim_i32,
        polynomial_coeff,
        &mut curve,
    )?;
    let slice: &[f64] = &curve[u_dim * u_request..];
    eval_polynomial(
        v,
        v_derivative_request,
        v_degree,
        dimension,
        slice,
        &mut point,
    )?;
    results[..dim].copy_from_slice(&point[dim * v_request..]);
    Ok(())
}

fn too_wide(name: &str, width: usize) -> Failure {
    Failure::out_of_range(&format!(
        "{name}: (VDegree+1)*Dimension = {width}; expected <= i32::MAX"
    ))
}
