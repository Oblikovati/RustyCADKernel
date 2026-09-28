// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Direct polynomial solutions preserving pinned MathPoly_Utils.hxx arithmetic.
use super::extended::{Extended as E, exponent, scale_double};
use super::{PolyResult, Status};
mod isolation;
use super::long_poly::{scaled_polynomial, small_residual};
use isolation::{distinct_real, multiplicity};
/// Solves a linear equation with native degenerate statuses; `linear(2.0,-4.0)`.
pub fn linear(a: f64, b: f64) -> PolyResult {
    solve_direct(&[b, a])
}
/// Uses the compensated native discriminant; `quadratic(1.0,-5.0,6.0)`.
pub fn quadratic(a: f64, b: f64, c: f64) -> PolyResult {
    solve_direct(&[c, b, a])
}
/// Isolates real roots recursively with x87 precision; `cubic(1.0,-6.0,11.0,-6.0)`.
pub fn cubic(a: f64, b: f64, c: f64, d: f64) -> PolyResult {
    solve_direct(&[d, c, b, a])
}
/// Preserves native real-root multiplicities; `quartic(1.0,-10.0,35.0,-50.0,24.0)`.
pub fn quartic(a: f64, b: f64, c: f64, d: f64, e: f64) -> PolyResult {
    solve_direct(&[e, d, c, b, a])
}
fn solve_direct(coefficients: &[f64]) -> PolyResult {
    if coefficients.iter().any(|x| !x.is_finite()) {
        return PolyResult::failed(Status::InvalidInput);
    }
    let degree: usize = coefficients.iter().rposition(|&x| x != 0.0).unwrap_or(0);
    match degree {
        0 => PolyResult::failed(if coefficients[0] == 0.0 {
            Status::InfiniteSolutions
        } else {
            Status::NoSolution
        }),
        1 => linear_result(coefficients),
        2 => quadratic_result(coefficients),
        _ => higher_result(&coefficients[..=degree]),
    }
}
fn linear_result(c: &[f64]) -> PolyResult {
    let root: Option<f64> = (-E::new(c[0]) / E::new(c[1])).double();
    let Some(root) = root else {
        return PolyResult::failed(Status::NumericalError);
    };
    let mut out: PolyResult = PolyResult::failed(Status::Ok);
    out.append(root, 1);
    out
}
fn quadratic_result(c: &[f64]) -> PolyResult {
    let maximum: f64 = c.iter().fold(0.0_f64, |s, x| s.max(x.abs()));
    let power: i32 = exponent(maximum);
    let a: f64 = scale_double(c[2], -power);
    let b: f64 = scale_double(c[1], -power);
    let c: f64 = scale_double(c[0], -power);
    let four_a: f64 = 4.0 * a;
    let four_ac: f64 = four_a * c;
    let product_error: f64 = (-four_a).mul_add(c, four_ac);
    let discriminant: f64 = b.mul_add(b, -four_ac) + product_error;
    if discriminant < 0.0 {
        return PolyResult::failed(Status::Ok);
    }
    if discriminant == 0.0 {
        return repeated_quadratic(-b / (2.0 * a));
    }
    let q: f64 = -0.5 * (b + discriminant.sqrt().copysign(b));
    distinct_quadratic(q / a, c / q)
}
fn repeated_quadratic(root: f64) -> PolyResult {
    if !root.is_finite() {
        return PolyResult::failed(Status::NumericalError);
    }
    let mut out: PolyResult = PolyResult::failed(Status::Ok);
    out.append(root, 2);
    out
}
fn distinct_quadratic(first: f64, second: f64) -> PolyResult {
    let mut out: PolyResult = PolyResult::failed(Status::Ok);
    for root in [first, second] {
        if !root.is_finite() {
            out.status = Status::NumericalError;
            out.nb_roots = 0;
            return out;
        }
        out.append(root, 1);
    }
    if out.roots[0] > out.roots[1] {
        out.roots.swap(0, 1);
    }
    out
}
fn higher_result(c: &[f64]) -> PolyResult {
    let (scaled, power) = scaled_polynomial(c);
    let Some(mut roots) = distinct_real(&scaled) else {
        return PolyResult::failed(Status::NumericalError);
    };
    roots.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut out: PolyResult = PolyResult::failed(Status::NotConverged);
    for root in roots {
        let count: usize = multiplicity(&scaled, root);
        let value: Option<f64> = root.scale(power).double();
        if !small_residual(&scaled, root, 256.0) || value.is_none() {
            out.status = Status::NumericalError;
            return out;
        }
        out.append(value.unwrap(), count);
    }
    out.status = Status::Ok;
    out
}
