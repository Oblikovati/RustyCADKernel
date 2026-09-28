// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Port of MathSys_NewtonTypes/2D/3D/4D at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::box_bounds::{bounds_valid, clamp, max_step};
use super::checks::{
    LINE_SEARCH_MAX, is_armijo_accepted, is_finite_array, is_step_within_tolerance,
    merit_directional_derivative, safe_distance_n, safe_norm_n,
};
use super::{NewtonBoundsN, NewtonOptions, NewtonResultN};
use crate::math_poly::extended::LongDouble as E;
use crate::math_utils::Status;
pub(super) trait Policy<const N: usize> {
    const SYMMETRIC: bool = false;
    fn system(&mut self, x: &[f64; N]) -> Result<([f64; N], [[f64; N]; N]), Status>;
    fn value_norm(&mut self, x: &[f64; N]) -> Result<f64, Status>;
    fn direction(&self, j: &[[f64; N]; N], f: &[f64; N], residual: f64) -> Option<[f64; N]>;
}
pub(super) fn solve<const N: usize, P: Policy<N>>(
    policy: &mut P,
    start: [f64; N],
    bounds: &NewtonBoundsN<N>,
    options: &NewtonOptions,
) -> NewtonResultN<N> {
    let mut result: NewtonResultN<N> = NewtonResultN {
        x: start,
        ..Default::default()
    };
    if !options.is_valid() || !bounds_valid(bounds) || !is_finite_array(&start) {
        result.status = Status::InvalidInput;
        return result;
    }
    clamp(
        &mut result.x,
        bounds,
        options.allow_soft_bounds,
        options.soft_bounds_extension,
    );
    for iteration in 0..options.solver.max_iterations {
        result.iterations = iteration + 1;
        match advance(policy, &mut result, bounds, options) {
            Ok(true) => continue,
            Ok(false) => {
                result.status = Status::Ok;
                return result;
            }
            Err(status) => {
                result.status = status;
                return result;
            }
        }
    }
    result.status = finish(policy, &mut result, options);
    result
}
fn advance<const N: usize, P: Policy<N>>(
    policy: &mut P,
    result: &mut NewtonResultN<N>,
    bounds: &NewtonBoundsN<N>,
    options: &NewtonOptions,
) -> Result<bool, Status> {
    let (f, j): ([f64; N], [[f64; N]; N]) = policy.system(&result.x)?;
    result.residual_norm = safe_norm_n(&f);
    if result.residual_norm <= options.solver.f_tolerance {
        return Ok(false);
    }
    let mut delta: [f64; N] = policy
        .direction(&j, &f, result.residual_norm)
        .ok_or(Status::Singular)?;
    limit_step(&mut delta, max_step(bounds, options.max_step_ratio));
    let (next, accepted_norm): ([f64; N], Option<f64>) =
        search(policy, result, &f, &j, &delta, bounds, options)?;
    let within: bool = is_step_within_tolerance(&next, &result.x, options.solver.x_tolerance);
    result.step_norm = safe_distance_n(&next, &result.x);
    result.x = next;
    if let Some(value) = accepted_norm {
        result.residual_norm = value;
    }
    if within {
        return Err(finish(policy, result, options));
    }
    Ok(true)
}
fn limit_step<const N: usize>(delta: &mut [f64; N], limit: Option<f64>) {
    let length: f64 = safe_norm_n(delta);
    if let Some(maximum) = limit.filter(|v| length > *v) {
        let scale: f64 = maximum / length;
        for value in delta {
            *value *= scale;
        }
    }
}
fn candidate<const N: usize>(
    x: &[f64; N],
    delta: &[f64; N],
    alpha: f64,
    bounds: &NewtonBoundsN<N>,
    options: &NewtonOptions,
) -> [f64; N] {
    let mut next: [f64; N] = std::array::from_fn(|i| x[i] + alpha * delta[i]);
    clamp(
        &mut next,
        bounds,
        options.allow_soft_bounds,
        options.soft_bounds_extension,
    );
    next
}
#[allow(clippy::too_many_arguments)]
fn search<const N: usize, P: Policy<N>>(
    policy: &mut P,
    result: &NewtonResultN<N>,
    f: &[f64; N],
    j: &[[f64; N]; N],
    delta: &[f64; N],
    bounds: &NewtonBoundsN<N>,
    options: &NewtonOptions,
) -> Result<([f64; N], Option<f64>), Status> {
    if !options.enable_line_search {
        return Ok((candidate(&result.x, delta, 1.0, bounds, options), None));
    }
    for iteration in 0..LINE_SEARCH_MAX as i32 {
        let next: [f64; N] = candidate(&result.x, delta, 2.0_f64.powi(-iteration), bounds, options);
        let step: [f64; N] = std::array::from_fn(|i| next[i] - result.x[i]);
        let derivative: E = merit_directional_derivative(f, j, &step);
        if P::SYMMETRIC && derivative.partial_cmp(&E::ZERO) != Some(std::cmp::Ordering::Less) {
            continue;
        }
        let Ok(norm) = policy.value_norm(&next) else {
            continue;
        };
        if is_armijo_accepted(result.residual_norm, norm, derivative) {
            return Ok((next, P::SYMMETRIC.then_some(norm)));
        }
    }
    Err(Status::NonDescentDirection)
}
fn finish<const N: usize, P: Policy<N>>(
    policy: &mut P,
    result: &mut NewtonResultN<N>,
    options: &NewtonOptions,
) -> Status {
    let norm: f64 = match policy.value_norm(&result.x) {
        Ok(value) => value,
        Err(status) => return status,
    };
    result.residual_norm = norm;
    if norm <= options.solver.f_tolerance {
        Status::Ok
    } else {
        Status::MaxIterations
    }
}
