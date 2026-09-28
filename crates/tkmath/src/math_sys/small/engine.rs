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
use super::{NewtonBoundsN, NewtonOptions, NewtonResultN, merit};
use crate::math_poly::extended::Extended as E;
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
    if !options.is_valid() || !bounds.is_valid() || !start.iter().all(|v| v.is_finite()) {
        result.status = Status::InvalidInput;
        return result;
    }
    bounds.clamp(&mut result.x, options);
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
    result.residual_norm = if P::SYMMETRIC {
        f[0].hypot(f[1])
    } else {
        merit::norm(&f)
    };
    if result.residual_norm <= options.solver.f_tolerance {
        return Ok(false);
    }
    let mut delta: [f64; N] = policy
        .direction(&j, &f, result.residual_norm)
        .ok_or(Status::Singular)?;
    limit_step(&mut delta, bounds.max_step(options.max_step_ratio));
    let (next, accepted_norm): ([f64; N], Option<f64>) =
        search(policy, result, &f, &j, &delta, bounds, options)?;
    let step: [f64; N] = std::array::from_fn(|i| next[i] - result.x[i]);
    let within: bool = step.iter().all(|v| {
        v.abs()
            .partial_cmp(&options.solver.x_tolerance)
            .is_none_or(|v| !v.is_gt())
    });
    result.step_norm = merit::norm(&step);
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
    let length: f64 = if N == 2 {
        delta[0].hypot(delta[1])
    } else {
        merit::norm(delta)
    };
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
    bounds.clamp(&mut next, options);
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
    for iteration in 0..8 {
        let next: [f64; N] = candidate(&result.x, delta, 2.0_f64.powi(-iteration), bounds, options);
        let step: [f64; N] = std::array::from_fn(|i| next[i] - result.x[i]);
        let derivative: E = merit::derivative(f, j, &step);
        if P::SYMMETRIC && derivative.partial_cmp(&E::ZERO) != Some(std::cmp::Ordering::Less) {
            continue;
        }
        let Ok(norm) = policy.value_norm(&next) else {
            continue;
        };
        if merit::armijo(result.residual_norm, norm, derivative) {
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
