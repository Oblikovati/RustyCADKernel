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

//! MathUtils_Poly.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Slice lengths encode degree plus one; an empty slice represents an invalid polynomial.

/// Evaluates constant-first coefficients; `eval_poly(&[1.0, 2.0], 3.0)` is 7.
pub fn eval_poly(coefficients: &[f64], x: f64) -> f64 {
    if coefficients.is_empty() || !x.is_finite() {
        return f64::NAN;
    }
    let mut value: f64 = coefficients[coefficients.len() - 1];
    for coefficient in coefficients[..coefficients.len() - 1].iter().rev() {
        value = value * x + coefficient;
    }
    value
}
/// Returns value and derivative together; `eval_poly_deriv(&[1.0, 2.0], 3.0)` is (7,2).
pub fn eval_poly_deriv(coefficients: &[f64], x: f64) -> (f64, f64) {
    if coefficients.is_empty() || !x.is_finite() {
        return (f64::NAN, f64::NAN);
    }
    let mut value: f64 = coefficients[coefficients.len() - 1];
    let mut derivative: f64 = 0.0;
    for coefficient in coefficients[..coefficients.len() - 1].iter().rev() {
        derivative = derivative * x + value;
        value = value * x + coefficient;
    }
    (value, derivative)
}
/// Evaluates leading-first coefficients; `eval_poly_desc(&[2.0, 1.0], 3.0)` is 7.
pub fn eval_poly_desc(coefficients: &[f64], x: f64) -> f64 {
    if coefficients.is_empty() || !x.is_finite() {
        return f64::NAN;
    }
    let mut value: f64 = coefficients[0];
    for coefficient in &coefficients[1..] {
        value = value * x + coefficient;
    }
    value
}
/// Polishes a root without changing native stopping rules; `refine_poly_root(&[-2.0,0.0,1.0],1.4,5)`.
pub fn refine_poly_root(coefficients: &[f64], root: f64, max_iterations: u32) -> f64 {
    if coefficients.is_empty() || !root.is_finite() {
        return f64::NAN;
    }
    let mut x: f64 = root;
    for _ in 0..max_iterations {
        let (value, derivative): (f64, f64) = eval_poly_deriv(coefficients, x);
        if !value.is_finite() || !derivative.is_finite() || derivative == 0.0 {
            break;
        }
        let delta: f64 = value / derivative;
        if !delta.is_finite() || !(x - delta).is_finite() {
            return f64::NAN;
        }
        x -= delta;
        if delta.abs() < f64::EPSILON * x.abs().max(1.0) {
            break;
        }
    }
    x
}
/// Refines descending coefficients of degree at most four; `refine_poly_root_desc(&[1.0,-2.0],1.0,5)`.
pub fn refine_poly_root_desc(coefficients: &[f64], root: f64, max_iterations: u32) -> f64 {
    if coefficients.is_empty() || coefficients.len() > 5 {
        return f64::NAN;
    }
    let mut ascending: [f64; 5] = [0.0; 5];
    for (index, coefficient) in coefficients.iter().rev().enumerate() {
        ascending[index] = *coefficient;
    }
    refine_poly_root(&ascending[..coefficients.len()], root, max_iterations)
}
/// Native insertion sort preserves unordered NaN comparisons; `sort_roots(&mut [2.0,1.0])`.
pub fn sort_roots(roots: &mut [f64]) {
    for index in 1..roots.len() {
        let key: f64 = roots[index];
        let mut position: usize = index;
        while position > 0 && roots[position - 1] > key {
            roots[position] = roots[position - 1];
            position -= 1;
        }
        roots[position] = key;
    }
}
/// Compacts sorted roots and returns retained count; `remove_duplicate_roots(&mut [1.0,1.0],1e-10)`.
pub fn remove_duplicate_roots(roots: &mut [f64], tolerance: f64) -> usize {
    if roots.is_empty() || !tolerance.is_finite() || tolerance < 0.0 || !roots[0].is_finite() {
        return 0;
    }
    let mut retained: usize = 1;
    for index in 1..roots.len() {
        if !roots[index].is_finite() {
            return 0;
        }
        let scale: f64 = roots[index].abs().max(roots[retained - 1].abs());
        if (roots[index] - roots[retained - 1]).abs() > tolerance + tolerance * scale {
            roots[retained] = roots[index];
            retained += 1;
        }
    }
    retained
}
/// Returns depressed coefficients (p,q) and shift; `depress_cubic(3.0,3.0,1.0)` is (0,0,1).
pub fn depress_cubic(b: f64, c: f64, d: f64) -> (f64, f64, f64) {
    let shift: f64 = b / 3.0;
    let squared: f64 = b * b;
    (
        c - squared / 3.0,
        d - b * c / 3.0 + 2.0 * squared * b / 27.0,
        shift,
    )
}
/// Returns depressed coefficients (p,q,r) and shift; `depress_quartic(4.0,6.0,4.0,1.0)`.
pub fn depress_quartic(b: f64, c: f64, d: f64, e: f64) -> (f64, f64, f64, f64) {
    let shift: f64 = b / 4.0;
    let squared: f64 = b * b;
    let cubed: f64 = squared * b;
    let fourth: f64 = squared * squared;
    (
        c - 3.0 * squared / 8.0,
        d - b * c / 2.0 + cubed / 8.0,
        e - b * d / 4.0 + squared * c / 16.0 - 3.0 * fourth / 256.0,
        shift,
    )
}
