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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_TBezierSurface.cxx (evalTrigAxisDerivs, evalTrigAxisNthDeriv, EvalD0..EvalDN) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! T-Bezier surface evaluation: the trigonometric basis of each axis, the tensor sums
//! and the rational quotients of orders 1 to 3.
use crate::geom_eval::bezier_grid::{
    AxisBasis, ORDER_PAIRS, PoleGrid, TensorSums, WeightProduct, pair_count, tensor_sums,
};
use crate::geom_eval::bezier_results::SurfaceDerivs;
use rustycad_tkg2d::geom2d_eval::eval_basis::{pow_int, trig_shift};
use rustycad_tkmath::gp::Xyz;

/// The poles, the optional weights and the frequencies of one surface.
#[derive(Clone, Copy)]
pub(super) struct TBezierSurfaceView<'a> {
    pub(super) grid: PoleGrid<'a>,
    pub(super) alpha_u: f64,
    pub(super) alpha_v: f64,
}

/// `evalTrigAxisDerivs<MAX>`: `1, sin(k a t), cos(k a t)` (k = 1..order) and their
/// derivatives up to MAX, with `freq = k * alpha` and `arg = freq * t`;
/// `axis_derivs(0.5, 1.0, 1, 1).at(1, 1)`.
pub(super) fn axis_derivs(t: f64, alpha: f64, order: usize, max: usize) -> AxisBasis {
    let dim: usize = 2 * order + 1;
    let mut basis = AxisBasis::zeros(max, dim);
    *basis.at_mut(0, 0) = 1.0;
    for k in 1..=order {
        let freq: f64 = k as f64 * alpha;
        let arg: f64 = freq * t;
        let (sin, cos) = (arg.sin(), arg.cos());
        let freq2: f64 = freq * freq;
        let freq3: f64 = freq2 * freq;
        let values: [[f64; 2]; 4] = [
            [sin, cos],
            [freq * cos, -freq * sin],
            [-freq2 * sin, -freq2 * cos],
            [-freq3 * cos, freq3 * sin],
        ];
        for (d, pair) in values.iter().enumerate().take(max + 1) {
            *basis.at_mut(d, 2 * k - 1) = pair[0];
            *basis.at_mut(d, 2 * k) = pair[1];
        }
    }
    basis
}

/// `evalTrigAxisNthDeriv`: the order-`n` derivative of every basis function, stored as
/// order 0 of the result; order 0 recomputes `sin((k * a) * t)` directly;
/// `axis_nth_deriv(0.5, 1.0, 1, 4).at(0, 1)`.
pub(super) fn axis_nth_deriv(t: f64, alpha: f64, order: usize, n: i32) -> AxisBasis {
    let mut basis = AxisBasis::zeros(0, 2 * order + 1);
    *basis.at_mut(0, 0) = if n == 0 { 1.0 } else { 0.0 };
    for k in 1..=order {
        let freq: f64 = k as f64 * alpha;
        let arg: f64 = freq * t;
        let (sin, cos) = (arg.sin(), arg.cos());
        let (value_sin, value_cos) = if n == 0 {
            (sin, cos)
        } else {
            let power: f64 = pow_int(freq, n);
            let (sin_n, cos_n) = trig_shift(sin, cos, n);
            (power * sin_n, power * cos_n)
        };
        *basis.at_mut(0, 2 * k - 1) = value_sin;
        *basis.at_mut(0, 2 * k) = value_cos;
    }
    basis
}

/// The tensor sums of the evaluator of order `max` at `(u, v)`;
/// `sums(&view, 0.5, 0.5, 2).n[5]`.
pub(super) fn sums(view: &TBezierSurfaceView<'_>, u: f64, v: f64, max: usize) -> TensorSums {
    let grid: &PoleGrid<'_> = &view.grid;
    let basis_u: AxisBasis = axis_derivs(u, view.alpha_u, (grid.rows - 1) / 2, max);
    let basis_v: AxisBasis = axis_derivs(v, view.alpha_v, (grid.cols - 1) / 2, max);
    let pairs = &ORDER_PAIRS[..pair_count(max)];
    tensor_sums(
        grid,
        [&basis_u, &basis_v],
        pairs,
        WeightProduct::WeightFirst,
    )
}

/// The non-rational `EvalDN`: the tensor sum of the order-`nu` and order-`nv` bases;
/// `non_rational_dn(&view, 0.5, 0.5, 2, 3)`.
pub(super) fn non_rational_dn(
    view: &TBezierSurfaceView<'_>,
    u: f64,
    v: f64,
    nu: i32,
    nv: i32,
) -> Xyz {
    let grid: &PoleGrid<'_> = &view.grid;
    let basis_u: AxisBasis = axis_nth_deriv(u, view.alpha_u, (grid.rows - 1) / 2, nu);
    let basis_v: AxisBasis = axis_nth_deriv(v, view.alpha_v, (grid.cols - 1) / 2, nv);
    let sums: TensorSums = tensor_sums(
        grid,
        [&basis_u, &basis_v],
        &[(0, 0)],
        WeightProduct::WeightFirst,
    );
    sums.n[0]
}

/// The rational quotients `S`, `S_u`, ... in `ORDER_PAIRS` order, for the first
/// `pair_count(max)` pairs. The native `2.0 * w_u * S_u` is `(2.0 * w_u) * S_u`,
/// written with the scalar on the right; `rational(&sums, 3)`.
pub(super) fn rational(sums: &TensorSums, max: usize) -> SurfaceDerivs {
    let (n, w) = (&sums.n, &sums.w);
    let mut s = [Xyz::default(); 10];
    s[0] = n[0] / w[0];
    if max >= 1 {
        s[1] = (n[1] - s[0] * w[1]) / w[0];
        s[2] = (n[2] - s[0] * w[2]) / w[0];
    }
    if max >= 2 {
        s[3] = (n[3] - s[1] * (2.0 * w[1]) - s[0] * w[3]) / w[0];
        s[4] = (n[4] - s[2] * (2.0 * w[2]) - s[0] * w[4]) / w[0];
        s[5] = (n[5] - s[2] * w[1] - s[1] * w[2] - s[0] * w[5]) / w[0];
    }
    if max >= 3 {
        rational_third(n, w, &mut s);
    }
    s
}

// S_uuu, S_vvv, then S_uuv and S_uvv from the Leibniz rule, term order as natively.
fn rational_third(n: &[Xyz], w: &[f64], s: &mut SurfaceDerivs) {
    s[6] = (n[6] - s[3] * (3.0 * w[1]) - s[1] * (3.0 * w[3]) - s[0] * w[6]) / w[0];
    s[7] = (n[7] - s[4] * (3.0 * w[2]) - s[2] * (3.0 * w[4]) - s[0] * w[7]) / w[0];
    s[8] = (n[8]
        - s[0] * w[8]
        - s[5] * (2.0 * w[1])
        - s[3] * w[2]
        - s[2] * w[3]
        - s[1] * (2.0 * w[5]))
        / w[0];
    s[9] = (n[9]
        - s[0] * w[9]
        - s[5] * (2.0 * w[2])
        - s[4] * w[1]
        - s[1] * w[4]
        - s[2] * (2.0 * w[5]))
        / w[0];
}
