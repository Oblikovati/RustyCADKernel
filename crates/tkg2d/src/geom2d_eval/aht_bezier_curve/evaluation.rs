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
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_AHTBezierCurve.cxx (evalPolynomialNonRational, evalPolynomialRational, evalMixedAHT, EvalD1..EvalD3) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! AHT-Bezier evaluation of orders 0..3: Horner sums for a purely algebraic basis,
//! direct basis sums for a mixed basis, and the rational quotients.
use super::super::basis_sums::{BasisDerivatives, BasisSums};
use rustycad_tkmath::gp::{Point2, Xy};

/// The poles, the optional weights and the basis parameters of one curve, as slices
/// in storage order (the native code indexes `&Value(Lower())` pointers).
#[derive(Clone, Copy)]
pub(super) struct AhtView<'a> {
    pub(super) poles: &'a [Point2],
    pub(super) weights: Option<&'a [f64]>,
    pub(super) alg_degree: usize,
    pub(super) alpha: f64,
    pub(super) beta: f64,
}

/// `evalPolynomialNonRational<MAX>` / `evalPolynomialRational<MAX>`: Horner's scheme on
/// `sum P_k t^k`, with its derivatives up to MAX; `polynomial::<1>(&view, 0.5)`.
pub(super) fn polynomial<const MAX: usize>(view: &AhtView<'_>, u: f64) -> BasisSums {
    let degree: usize = view.alg_degree;
    let mut s = BasisSums::default();
    let weight = |k: usize| view.weights.map_or(1.0, |w| w[k]);
    s.n[0] = match view.weights {
        None => view.poles[degree].xy(),
        Some(_) => view.poles[degree].xy() * weight(degree),
    };
    s.w[0] = view.weights.map_or(0.0, |_| weight(degree));
    for k in (0..degree).rev() {
        horner_derivative_steps::<MAX>(&mut s, u, view.weights.is_some());
        s.n[0] = match view.weights {
            None => s.n[0] * u + view.poles[k].xy(),
            Some(_) => s.n[0] * u + view.poles[k].xy() * weight(k),
        };
        if view.weights.is_some() {
            s.w[0] = s.w[0] * u + weight(k);
        }
    }
    s
}

// `N3 = N3 u + 3 N2`, `N2 = N2 u + 2 N1`, `N1 = N1 u + N0` (and the W twins), highest first.
fn horner_derivative_steps<const MAX: usize>(s: &mut BasisSums, u: f64, rational: bool) {
    for d in (1..=MAX.min(3)).rev() {
        let factor: f64 = d as f64;
        s.n[d] = if d == 1 {
            s.n[1] * u + s.n[0]
        } else {
            s.n[d] * u + s.n[d - 1] * factor
        };
        if rational {
            s.w[d] = if d == 1 {
                s.w[1] * u + s.w[0]
            } else {
                s.w[d] * u + s.w[d - 1] * factor
            };
        }
    }
}

// The algebraic basis `t^k` and its derivatives 1..3 from the running powers.
fn power_derivatives(k: usize, powers: &[f64; 4]) -> BasisDerivatives {
    let kf: f64 = k as f64;
    let b1: f64 = if k >= 1 { kf * powers[1] } else { 0.0 };
    let b2: f64 = if k >= 2 {
        kf * (k - 1) as f64 * powers[2]
    } else {
        0.0
    };
    let b3: f64 = if k >= 3 {
        kf * (k - 1) as f64 * (k - 2) as f64 * powers[3]
    } else {
        0.0
    };
    [powers[0], b1, b2, b3]
}

/// `evalMixedAHT<MAX, rational>`: the algebraic part `1, t, ..., t^k`, then the
/// `sinh/cosh(alpha t)` pair when alpha > 0, then the `sin/cos(beta t)` pair when
/// beta > 0; `mixed::<2>(&view, 0.5)`.
pub(super) fn mixed<const MAX: usize>(view: &AhtView<'_>, u: f64) -> BasisSums {
    let mut s = BasisSums::default();
    // powers = [t^k, t^(k-1), t^(k-2), t^(k-3)], shifted after each term.
    let mut powers: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    for k in 0..=view.alg_degree {
        let weight: Option<f64> = view.weights.map(|w| w[k]);
        s.add_term::<MAX>(view.poles[k].xy(), weight, &power_derivatives(k, &powers));
        powers = [powers[0] * u, powers[0], powers[1], powers[2]];
    }
    let mut index: usize = view.alg_degree + 1;
    if view.alpha > 0.0 {
        add_pair_at::<MAX>(&mut s, view, index, hyperbolic_pair(view.alpha, u));
        index += 2;
    }
    if view.beta > 0.0 {
        add_pair_at::<MAX>(&mut s, view, index, trigonometric_pair(view.beta, u));
    }
    s
}

fn add_pair_at<const MAX: usize>(
    s: &mut BasisSums,
    view: &AhtView<'_>,
    index: usize,
    basis: [BasisDerivatives; 2],
) {
    let poles: [Xy; 2] = [view.poles[index].xy(), view.poles[index + 1].xy()];
    let weights: Option<[f64; 2]> = view.weights.map(|w| [w[index], w[index + 1]]);
    s.add_pair::<MAX>(poles, weights, [&basis[0], &basis[1]]);
}

// sinh(a t), cosh(a t) and their derivatives 1..3.
fn hyperbolic_pair(alpha: f64, u: f64) -> [BasisDerivatives; 2] {
    let (sinh, cosh) = ((alpha * u).sinh(), (alpha * u).cosh());
    let alpha2: f64 = alpha * alpha;
    let alpha3: f64 = alpha2 * alpha;
    [
        [sinh, alpha * cosh, alpha2 * sinh, alpha3 * cosh],
        [cosh, alpha * sinh, alpha2 * cosh, alpha3 * sinh],
    ]
}

// sin(b t), cos(b t) and their derivatives 1..3.
fn trigonometric_pair(beta: f64, u: f64) -> [BasisDerivatives; 2] {
    let (sin, cos) = ((beta * u).sin(), (beta * u).cos());
    let beta2: f64 = beta * beta;
    let beta3: f64 = beta2 * beta;
    [
        [sin, beta * cos, -beta2 * sin, -beta3 * cos],
        [cos, -beta * sin, -beta2 * cos, beta3 * sin],
    ]
}

/// The rational point and first derivative of EvalD1 and EvalD2:
/// `C = N0 / W0` and `C' = (N1 W0 - N0 W1) / W0^2`, through `1 / W0`;
/// `quotient_d1(&sums)`.
pub(super) fn quotient_d1(s: &BasisSums) -> (Xy, Xy) {
    let inv_w: f64 = 1.0 / s.w[0];
    let c: Xy = s.n[0] * inv_w;
    let d1: Xy = (s.n[1] * s.w[0] - s.n[0] * s.w[1]) * (inv_w * inv_w);
    (c, d1)
}

/// The rational second derivative from the point and first derivative;
/// `quotient_d2(&sums, c, d1)`.
pub(super) fn quotient_d2(s: &BasisSums, c: Xy, d1: Xy) -> Xy {
    let inv_w: f64 = 1.0 / s.w[0];
    (s.n[2] - d1 * (2.0 * s.w[1]) - c * s.w[2]) * inv_w
}

/// EvalD3's rational chain: unlike EvalD1 and EvalD2 it computes `C' = (N1 - C W1) / W0`,
/// so its first derivative can differ from theirs in the last bits;
/// `quotient_d3(&sums)`.
pub(super) fn quotient_d3(s: &BasisSums) -> [Xy; 4] {
    let inv_w: f64 = 1.0 / s.w[0];
    let c: Xy = s.n[0] * inv_w;
    let d1: Xy = (s.n[1] - c * s.w[1]) * inv_w;
    let d2: Xy = quotient_d2(s, c, d1);
    let d3: Xy = (s.n[3] - d2 * (3.0 * s.w[1]) - d1 * (3.0 * s.w[2]) - c * s.w[3]) * inv_w;
    [c, d1, d2, d3]
}
