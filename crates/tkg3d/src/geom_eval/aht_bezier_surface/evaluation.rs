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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_AHTBezierSurface.cxx (EvalD0..EvalDN, evalTensorDerivs, computeRationalDerivs) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! AHT-Bezier surface evaluation: the Horner path of a purely algebraic, non-rational
//! surface, otherwise the tensor sums of the axis bases, with the explicit rational
//! quotients up to order 3 and the Leibniz recurrence above.
use super::axis::{AxisShape, axis_derivs};
use super::polynomial::polynomial;
use crate::geom_eval::bezier_grid::{
    AxisBasis, ORDER_PAIRS, PoleGrid, TensorSums, WeightProduct, full_tensor, tensor_sums,
};
use crate::geom_eval::bezier_results::SurfaceDerivs;
use rustycad_tkmath::gp::Xyz;

/// The poles, the optional weights and the basis of each axis of one surface.
#[derive(Clone, Copy)]
pub(super) struct AhtSurfaceView<'a> {
    pub(super) grid: PoleGrid<'a>,
    pub(super) axes: [AxisShape; 2],
}
impl AhtSurfaceView<'_> {
    // No weights and no hyperbolic or trigonometric pair in either direction.
    fn is_pure_polynomial(&self) -> bool {
        let [u, v] = self.axes;
        self.grid.weights.is_none()
            && u.alpha <= 0.0
            && u.beta <= 0.0
            && v.alpha <= 0.0
            && v.beta <= 0.0
    }
    // The axis bases up to orders `max_u` and `max_v`.
    fn bases(&self, u: f64, v: f64, [max_u, max_v]: [usize; 2]) -> [AxisBasis; 2] {
        let [shape_u, shape_v] = self.axes;
        [
            axis_derivs(u, max_u, shape_u, self.grid.rows),
            axis_derivs(v, max_v, shape_v, self.grid.cols),
        ]
    }
}

/// EvalD0..EvalD3 of order `max`, in `SurfaceDerivs` order; `evaluate(&view, 0.5, 0.5, 2)`.
pub(super) fn evaluate(view: &AhtSurfaceView<'_>, u: f64, v: f64, max: usize) -> SurfaceDerivs {
    if view.is_pure_polynomial() {
        let degrees = view.axes.map(|axis| axis.alg_degree);
        return polynomial(&view.grid, degrees, u, v, max);
    }
    let [basis_u, basis_v] = view.bases(u, v, [max, max]);
    let orders: Vec<(usize, usize)> = full_tensor(max, max);
    let product = WeightProduct::BasisFirst;
    let sums: TensorSums = tensor_sums(&view.grid, [&basis_u, &basis_v], &orders, product);
    let at = |du: usize, dv: usize| du * (max + 1) + dv;
    if view.grid.weights.is_none() {
        let mut s: SurfaceDerivs = [Xyz::default(); 10];
        for (k, &(du, dv)) in ORDER_PAIRS.iter().enumerate() {
            if du <= max && dv <= max && du + dv <= max {
                s[k] = sums.n[at(du, dv)];
            }
        }
        return s;
    }
    quotients(&sums, max)
}

/// The rational quotients of EvalD1..EvalD3, through `1 / W00`, term order as natively;
/// `quotients(&sums, 3)`.
fn quotients(sums: &TensorSums, max: usize) -> SurfaceDerivs {
    let n = |du: usize, dv: usize| sums.n[du * (max + 1) + dv];
    let w = |du: usize, dv: usize| sums.w[du * (max + 1) + dv];
    let inv_w: f64 = 1.0 / w(0, 0);
    let mut c: SurfaceDerivs = [Xyz::default(); 10];
    c[0] = n(0, 0) * inv_w;
    if max >= 1 {
        c[1] = (n(1, 0) - c[0] * w(1, 0)) * inv_w;
        c[2] = (n(0, 1) - c[0] * w(0, 1)) * inv_w;
    }
    if max >= 2 {
        c[3] = (n(2, 0) - c[1] * (2.0 * w(1, 0)) - c[0] * w(2, 0)) * inv_w;
        c[4] = (n(0, 2) - c[2] * (2.0 * w(0, 1)) - c[0] * w(0, 2)) * inv_w;
        c[5] = (n(1, 1) - c[1] * w(0, 1) - c[2] * w(1, 0) - c[0] * w(1, 1)) * inv_w;
    }
    if max >= 3 {
        c[6] = (n(3, 0) - c[3] * (3.0 * w(1, 0)) - c[1] * (3.0 * w(2, 0)) - c[0] * w(3, 0)) * inv_w;
        c[7] = (n(0, 3) - c[4] * (3.0 * w(0, 1)) - c[2] * (3.0 * w(0, 2)) - c[0] * w(0, 3)) * inv_w;
        c[8] = (n(2, 1) - c[5] * (2.0 * w(1, 0)) - c[3] * w(0, 1) - c[2] * w(2, 0)
            - c[1] * (2.0 * w(1, 1))
            - c[0] * w(2, 1))
            * inv_w;
        c[9] = (n(1, 2) - c[5] * (2.0 * w(0, 1)) - c[4] * w(1, 0) - c[1] * w(0, 2)
            - c[2] * (2.0 * w(1, 1))
            - c[0] * w(1, 2))
            * inv_w;
    }
    c
}

/// `EvalDN` above order 3: the direct basis product for a non-rational surface (the
/// Horner path is not used here), the full tensor and `computeRationalDerivs`
/// otherwise; `high_order_dn(&view, 0.5, 0.5, 2, 2)`.
pub(super) fn high_order_dn(view: &AhtSurfaceView<'_>, u: f64, v: f64, nu: usize, nv: usize) -> Xyz {
    let [basis_u, basis_v] = view.bases(u, v, [nu, nv]);
    let bases = [&basis_u, &basis_v];
    if view.grid.weights.is_none() {
        let sums = tensor_sums(&view.grid, bases, &[(nu, nv)], WeightProduct::BasisFirst);
        return sums.n[0];
    }
    let sums = tensor_sums(&view.grid, bases, &full_tensor(nu, nv), WeightProduct::BasisFirst);
    rational_derivatives(&sums, nu, nv)[nu * (nv + 1) + nv]
}

/// `computeRationalDerivs`: `C(du, dv) = (N(du, dv) - sum C(p,du) C(q,dv) W(p, q)
/// C(du - p, dv - q)) / W(0, 0)` by increasing total order, with the binomials
/// rebuilt as `bin * (n - m) / (m + 1)`; `rational_derivatives(&sums, 2, 2)`.
fn rational_derivatives(sums: &TensorSums, max_u: usize, max_v: usize) -> Vec<Xyz> {
    let at = |du: usize, dv: usize| du * (max_v + 1) + dv;
    let inv_w: f64 = 1.0 / sums.w[at(0, 0)];
    let mut c: Vec<Xyz> = vec![Xyz::default(); sums.n.len()];
    for total in 0..=max_u + max_v {
        for du in 0..=total.min(max_u) {
            let dv: usize = total - du;
            if dv > max_v {
                continue;
            }
            let sum: Xyz = leibniz_sum(sums, &c, (du, dv), max_v);
            c[at(du, dv)] = sum * inv_w;
        }
    }
    c
}

fn binomial(n: usize, k: usize) -> f64 {
    let mut value: f64 = 1.0;
    for m in 0..k {
        value = value * (n - m) as f64 / (m + 1) as f64;
    }
    value
}

// `N(du, dv) - sum_{(p, q) != (0, 0)} C(du,p) C(dv,q) W(p, q) C(du - p, dv - q)`.
fn leibniz_sum(sums: &TensorSums, c: &[Xyz], (du, dv): (usize, usize), max_v: usize) -> Xyz {
    let at = |a: usize, b: usize| a * (max_v + 1) + b;
    let mut sum: Xyz = sums.n[at(du, dv)];
    for p in 0..=du {
        let bin_u: f64 = binomial(du, p);
        for q in 0..=dv {
            if p == 0 && q == 0 {
                continue;
            }
            let coefficient: f64 = bin_u * binomial(dv, q) * sums.w[at(p, q)];
            sum -= c[at(du - p, dv - q)] * coefficient;
        }
    }
    sum
}
