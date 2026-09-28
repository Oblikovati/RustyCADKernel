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

//! `Utils::HasMultiplicity` and `Utils::HasComplexMultiplicity`: Newton refinement on the
//! (m-1)-th derivative, then a long-double check that derivatives 0..m-1 vanish.
use super::COMPUTATIONAL;
use crate::math_poly::Complex;
use crate::math_poly::complex::long::LongComplex;
use crate::math_poly::extended::Extended as E;
use crate::math_poly::long_poly::{derivative, evaluated};
/// Iteration cap for refining a candidate through a multiple derivative root.
const MULTIPLICITY_REFINEMENT_MAX_ITERATIONS: u32 = 64;
fn widened(c: &[f64]) -> Vec<E> {
    c.iter().map(|&x| E::new(x)).collect()
}
fn derivative_of_order(c: &[E], order: usize) -> Vec<E> {
    (0..order).fold(c.to_vec(), |current, _| derivative(&current))
}
/// Refines `root` in place when it is a root of multiplicity `m` of `c`.
pub(super) fn has_multiplicity(c: &[f64], root: &mut f64, m: usize) -> bool {
    let polynomial: Vec<E> = widened(c);
    let Some(refined) = refined_real(&derivative_of_order(&polynomial, m - 1), E::new(*root))
    else {
        return false;
    };
    let mut current: Vec<E> = polynomial;
    for _ in 0..m {
        let value: E = evaluated(&current, refined).0;
        let norm: E = current
            .iter()
            .rev()
            .skip(1)
            .fold(current.last().unwrap().abs(), |s, x| s + x.abs());
        if !value.finite() || value.abs() > E::new(COMPUTATIONAL) * E::new(1.0).max(norm) {
            return false;
        }
        current = derivative(&current);
    }
    *root = refined.narrow();
    true
}
/// Newton iteration; `None` when a value, derivative or iterate stops being finite.
fn refined_real(c: &[E], mut root: E) -> Option<E> {
    for _ in 0..MULTIPLICITY_REFINEMENT_MAX_ITERATIONS {
        let (value, slope, _) = evaluated(c, root);
        if !value.finite() || !slope.finite() {
            return None;
        }
        if slope == E::ZERO {
            break;
        }
        let next: E = root - value / slope;
        if !next.finite() {
            return None;
        }
        if next == root {
            break;
        }
        root = next;
    }
    Some(root)
}
/// Complex Horner value and derivative with GCC `__mulxc3` products.
fn evaluated_complex(c: &[E], x: LongComplex) -> (LongComplex, LongComplex) {
    let mut value: LongComplex = LongComplex::real(*c.last().unwrap());
    let mut slope: LongComplex = LongComplex::real(E::ZERO);
    for &coefficient in c[..c.len() - 1].iter().rev() {
        slope = slope * x + value;
        value = (value * x).plus_real(coefficient);
    }
    (value, slope)
}
fn refined_complex(c: &[E], mut root: LongComplex) -> Option<LongComplex> {
    for _ in 0..MULTIPLICITY_REFINEMENT_MAX_ITERATIONS {
        let (value, slope) = evaluated_complex(c, root);
        if !value.finite() || !slope.finite() {
            return None;
        }
        if slope.abs() == E::ZERO {
            break;
        }
        let next: LongComplex = root - value / slope;
        if !next.finite() {
            return None;
        }
        if next == root {
            break;
        }
        root = next;
    }
    Some(root)
}
/// Absolute value bound `sum |c_i| |x|^i`, evaluated in native Horner order.
fn complex_magnitude(c: &[E], abs_root: E) -> E {
    let leading: E = c.last().unwrap().abs();
    c[..c.len() - 1]
        .iter()
        .rev()
        .fold(leading, |m, x| m * abs_root + x.abs())
}
/// Complex counterpart of [`has_multiplicity`]; refines `root` in place on success.
pub(super) fn has_complex_multiplicity(c: &[f64], root: &mut Complex, m: usize) -> bool {
    let polynomial: Vec<E> = widened(c);
    let start: LongComplex = LongComplex::new(E::new(root.real()), E::new(root.imag()));
    let Some(refined) = refined_complex(&derivative_of_order(&polynomial, m - 1), start) else {
        return false;
    };
    let mut current: Vec<E> = polynomial;
    for _ in 0..m {
        let value: LongComplex = evaluated_complex(&current, refined).0;
        let magnitude: E = complex_magnitude(&current, refined.abs());
        if !value.finite() || value.abs() > E::new(COMPUTATIONAL) * E::new(1.0).max(magnitude) {
            return false;
        }
        current = derivative(&current);
    }
    *root = Complex::new(refined.re.narrow(), refined.im.narrow());
    true
}
