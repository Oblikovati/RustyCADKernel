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
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_AHTBezierCurve.cxx and ModelingData/TKG3d/GeomEval/GeomEval_AHTBezierCurve.cxx (EvalDN above order 3) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dEval_AHTBezierCurve::EvalDN` for N > 3: the basis differentiated N times
//! for a non-rational curve, and the Leibniz recurrence
//! `C^(d) = (N^(d) - sum_j C(d,j) W^(j) C^(d-j)) / W` for a rational one.
use super::aht::AhtView;
use super::{EvalCoord, EvalPole, falling_factorial, pow_int, trig_shift};

// The d-th derivative of t^k: 0 when d > k; at t = 0 only d == k survives (the native
// code avoids powInt(0, 0) ambiguity by testing U == 0 explicitly).
fn power_derivative(k: i32, d: i32, u: f64) -> f64 {
    if d > k {
        return 0.0;
    }
    if u == 0.0 {
        return if d == k { falling_factorial(k, d) } else { 0.0 };
    }
    falling_factorial(k, d) * pow_int(u, k - d)
}

// d-th derivatives of (sinh, cosh) given alpha^d: even orders keep the pair, odd swap it.
fn hyperbolic_derivative(alpha_pow: f64, sinh: f64, cosh: f64, d: i32) -> (f64, f64) {
    if d % 2 == 0 {
        (alpha_pow * sinh, alpha_pow * cosh)
    } else {
        (alpha_pow * cosh, alpha_pow * sinh)
    }
}

/// Non-rational `EvalDN`; `non_rational_dn(&view, 0.5, 4)`.
pub fn non_rational_dn<P: EvalPole>(view: &AhtView<'_, P>, u: f64, n: i32) -> P::Coord {
    let mut sum = P::Coord::default();
    for k in 0..=view.alg_degree {
        sum += view.poles[k].coord() * power_derivative(k as i32, n, u);
    }
    let mut index: usize = view.alg_degree + 1;
    if view.alpha > 0.0 {
        let (sinh, cosh) = ((view.alpha * u).sinh(), (view.alpha * u).cosh());
        let (sinh_n, cosh_n) = hyperbolic_derivative(pow_int(view.alpha, n), sinh, cosh, n);
        sum += view.poles[index].coord() * sinh_n;
        sum += view.poles[index + 1].coord() * cosh_n;
        index += 2;
    }
    if view.beta > 0.0 {
        let beta_pow: f64 = pow_int(view.beta, n);
        let (sin_n, cos_n) = trig_shift((view.beta * u).sin(), (view.beta * u).cos(), n);
        sum += view.poles[index].coord() * (beta_pow * sin_n);
        sum += view.poles[index + 1].coord() * (beta_pow * cos_n);
    }
    sum
}

// The pair poles and weights of a rational curve, with sinh/cosh and sin/cos at u
// (all zero for an absent pair, as the native locals).
struct RationalPairs<P: EvalPole> {
    hyperbolic: Option<([P::Coord; 2], [f64; 2])>,
    trigonometric: Option<([P::Coord; 2], [f64; 2])>,
    sinh_cosh: (f64, f64),
    sin_cos: (f64, f64),
}
impl<P: EvalPole> RationalPairs<P> {
    fn new(view: &AhtView<'_, P>, weights: &[f64], u: f64) -> Self {
        let pair = |index: usize| {
            let poles: [P::Coord; 2] = [view.poles[index].coord(), view.poles[index + 1].coord()];
            (poles, [weights[index], weights[index + 1]])
        };
        let hyper_index: usize = view.alg_degree + 1;
        let trig_index: usize = hyper_index + if view.alpha > 0.0 { 2 } else { 0 };
        let (alpha_u, beta_u) = (view.alpha * u, view.beta * u);
        Self {
            hyperbolic: (view.alpha > 0.0).then(|| pair(hyper_index)),
            trigonometric: (view.beta > 0.0).then(|| pair(trig_index)),
            sinh_cosh: if view.alpha > 0.0 {
                (alpha_u.sinh(), alpha_u.cosh())
            } else {
                (0.0, 0.0)
            },
            sin_cos: if view.beta > 0.0 {
                (beta_u.sin(), beta_u.cos())
            } else {
                (0.0, 0.0)
            },
        }
    }
    // Adds the d-th derivative of both pairs to `(N^(d), W^(d))`.
    fn add_derivative(
        &self,
        d: i32,
        alpha_pow: f64,
        beta_pow: f64,
        nd: &mut P::Coord,
        wd: &mut f64,
    ) {
        if let Some(([ps, pc], [ws, wc])) = self.hyperbolic {
            let (sinh, cosh) = self.sinh_cosh;
            let (d_sinh, d_cosh) = hyperbolic_derivative(alpha_pow, sinh, cosh, d);
            *nd += ps * (ws * d_sinh) + pc * (wc * d_cosh);
            *wd += ws * d_sinh + wc * d_cosh;
        }
        if let Some(([ps, pc], [ws, wc])) = self.trigonometric {
            let (d_sin, d_cos) = trig_shift(self.sin_cos.0, self.sin_cos.1, d);
            *nd += ps * (ws * beta_pow * d_sin) + pc * (wc * beta_pow * d_cos);
            *wd += ws * beta_pow * d_sin + wc * beta_pow * d_cos;
        }
    }
}

// `(N^(d), W^(d))`: the algebraic terms, then the pairs.
fn weighted_derivative<P: EvalPole>(
    view: &AhtView<'_, P>,
    weights: &[f64],
    pairs: &RationalPairs<P>,
    u: f64,
    d: i32,
    powers: (f64, f64),
) -> (P::Coord, f64) {
    let (mut nd, mut wd) = (P::Coord::default(), 0.0_f64);
    let algebraic = view.poles.iter().zip(weights).take(view.alg_degree + 1);
    for (k, (pole, &weight)) in algebraic.enumerate() {
        let basis: f64 = power_derivative(k as i32, d, u);
        nd += pole.coord() * (weight * basis);
        wd += weight * basis;
    }
    pairs.add_derivative(d, powers.0, powers.1, &mut nd, &mut wd);
    (nd, wd)
}

/// Rational `EvalDN`; `rational_dn(&view, weights, 0.5, 4)`.
pub fn rational_dn<P: EvalPole>(
    view: &AhtView<'_, P>,
    weights: &[f64],
    u: f64,
    n: i32,
) -> P::Coord {
    let pairs = RationalPairs::new(view, weights, u);
    let mut w_derivs: Vec<f64> = Vec::with_capacity(n as usize + 1);
    let mut c_derivs: Vec<P::Coord> = Vec::with_capacity(n as usize + 1);
    let (mut alpha_pow, mut beta_pow, mut inv_w0) = (1.0_f64, 1.0_f64, 0.0_f64);
    for d in 0..=n {
        let (nd, wd) = weighted_derivative(view, weights, &pairs, u, d, (alpha_pow, beta_pow));
        w_derivs.push(wd);
        if d == 0 {
            inv_w0 = 1.0 / wd;
            c_derivs.push(nd * inv_w0);
        } else {
            c_derivs.push(leibniz_step(nd, &w_derivs, &c_derivs, d as usize) * inv_w0);
        }
        alpha_pow *= view.alpha;
        beta_pow *= view.beta;
    }
    c_derivs[n as usize]
}

// `N^(d) - sum_j C(d,j) W^(j) C^(d-j)`, the binomial built as `binom * (d-j+1) / j`.
fn leibniz_step<C: EvalCoord>(nd: C, w_derivs: &[f64], c_derivs: &[C], d: usize) -> C {
    let (mut sum, mut binom) = (nd, 1.0_f64);
    for j in 1..=d {
        binom = binom * (d - j + 1) as f64 / j as f64;
        sum -= c_derivs[d - j] * (binom * w_derivs[j]);
    }
    sum
}
