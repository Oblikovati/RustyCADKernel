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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_AHTBezierSurface.cxx (evalAxisDerivs) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `evalAxisDerivs`: the AHT basis `1, t, ..., t^k, sinh(a t), cosh(a t), sin(b t),
//! cos(b t)` of one axis and its derivatives up to a given order.
use crate::geom_eval::bezier_grid::AxisBasis;
use rustycad_tkg2d::geom2d_eval::eval_basis::trig_shift;

/// The basis parameters of one axis (`AlgDegree`, `Alpha`, `Beta` in U or V).
#[derive(Clone, Copy)]
pub(super) struct AxisShape {
    pub(super) alg_degree: usize,
    pub(super) alpha: f64,
    pub(super) beta: f64,
}

/// The derivatives of orders `0..=max` of every basis function of `shape` at `t`,
/// stored order-major over `dim` functions; `axis_derivs(0.5, 2, shape, 5).at(1, 1)`.
pub(super) fn axis_derivs(t: f64, max: usize, shape: AxisShape, dim: usize) -> AxisBasis {
    let mut basis = AxisBasis::zeros(max, dim);
    if t == 0.0 {
        algebraic_at_zero(&mut basis, max, shape.alg_degree);
    } else {
        algebraic(&mut basis, t, max, shape.alg_degree);
    }
    let mut index: usize = shape.alg_degree + 1;
    if shape.alpha > 0.0 {
        hyperbolic(&mut basis, t, max, shape.alpha, index);
        index += 2;
    }
    if shape.beta > 0.0 {
        trigonometric(&mut basis, t, max, shape.beta, index);
    }
    basis
}

// At t = 0 only the order-k derivative of t^k survives, equal to k!.
fn algebraic_at_zero(basis: &mut AxisBasis, max: usize, degree: usize) {
    let mut factorial: f64 = 1.0;
    for k in 0..=degree {
        if k > 0 {
            factorial *= k as f64;
        }
        for d in 0..=max {
            *basis.at_mut(d, k) = if d == k { factorial } else { 0.0 };
        }
    }
}

// d^d t^k: t^k, then each order multiplies by `(k - d) / t` (the quotient first).
fn algebraic(basis: &mut AxisBasis, t: f64, max: usize, degree: usize) {
    let mut power: f64 = 1.0;
    for k in 0..=degree {
        let top: usize = k.min(max);
        let mut value: f64 = power;
        for d in 0..=top {
            *basis.at_mut(d, k) = value;
            if d < top {
                value *= (k - d) as f64 / t;
            }
        }
        power *= t;
    }
}

// Even orders keep (sinh, cosh), odd orders swap them; alpha^d is a running product.
fn hyperbolic(basis: &mut AxisBasis, t: f64, max: usize, alpha: f64, index: usize) {
    let alpha_t: f64 = alpha * t;
    let (sinh, cosh) = (alpha_t.sinh(), alpha_t.cosh());
    let mut alpha_pow: f64 = 1.0;
    for d in 0..=max {
        let (first, second) = if d % 2 == 0 {
            (sinh, cosh)
        } else {
            (cosh, sinh)
        };
        *basis.at_mut(d, index) = alpha_pow * first;
        *basis.at_mut(d, index + 1) = alpha_pow * second;
        alpha_pow *= alpha;
    }
}

// The (sin, cos) pair shifted by d quarter turns, times the running beta^d.
fn trigonometric(basis: &mut AxisBasis, t: f64, max: usize, beta: f64, index: usize) {
    let beta_t: f64 = beta * t;
    let (sin, cos) = (beta_t.sin(), beta_t.cos());
    let mut beta_pow: f64 = 1.0;
    for d in 0..=max {
        let (sin_d, cos_d) = trig_shift(sin, cos, d as i32);
        *basis.at_mut(d, index) = beta_pow * sin_d;
        *basis.at_mut(d, index + 1) = beta_pow * cos_d;
        beta_pow *= beta;
    }
}
