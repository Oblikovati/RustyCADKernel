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
// Rust translation of CurveBasisEval (Geom2dEval_TBezierCurve.cxx, GeomEval_TBezierCurve.cxx) and PolyEvalData (Geom2dEval_AHTBezierCurve.cxx, GeomEval_AHTBezierCurve.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The weighted sums `N_d = sum w_i P_i B_i^(d)` and `W_d = sum w_i B_i^(d)` (d = 0..3)
//! that the T-Bezier and AHT-Bezier evaluators accumulate, and the rational quotients
//! built from them.
//!
//! `MAX` is the native `theMaxOrder` template parameter: the orders above it are
//! never touched, so they keep their zero start exactly as natively.
use super::EvalCoord;

/// `N0..N3` and `W0..W3`, zero-initialized; `BasisSums::<Xy>::default().n[0]`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BasisSums<C: EvalCoord> {
    /// The weighted pole sums `N0..N3`.
    pub n: [C; 4],
    /// The weight sums `W0..W3` (zero for a non-rational curve).
    pub w: [f64; 4],
}

/// The derivative values of one basis function for orders 0..3; unused orders stay 0.
pub type BasisDerivatives = [f64; 4];

impl<C: EvalCoord> BasisSums<C> {
    /// Add one basis pair (a sin/cos or sinh/cosh pair) for orders `0..=MAX`.
    ///
    /// Without weights the native non-rational form `Ps * Bs + Pc * Bc` is used; with
    /// weights `Ps * (ws * Bs) + Pc * (wc * Bc)` and `W += ws * Bs + wc * Bc`.
    /// (The AHT non-rational path multiplies by unit weights, which is exact.)
    /// `sums.add_pair::<1>([ps, pc], None, [&bs, &bc])`.
    pub fn add_pair<const MAX: usize>(
        &mut self,
        poles: [C; 2],
        weights: Option<[f64; 2]>,
        basis: [&BasisDerivatives; 2],
    ) {
        let pairs = basis[0].iter().zip(basis[1].iter());
        for (d, (&bs, &bc)) in pairs.enumerate().take(MAX + 1) {
            match weights {
                None => self.n[d] += poles[0] * bs + poles[1] * bc,
                Some([ws, wc]) => {
                    self.n[d] += poles[0] * (ws * bs) + poles[1] * (wc * bc);
                    self.w[d] += ws * bs + wc * bc;
                }
            }
        }
    }
    /// Add one single basis function (an algebraic `t^k`) for orders `0..=MAX`:
    /// `N += P * B` or `N += P * (w * B)` and `W += w * B`;
    /// `sums.add_term::<2>(pole, Some(w), &b)`.
    pub fn add_term<const MAX: usize>(
        &mut self,
        pole: C,
        weight: Option<f64>,
        basis: &BasisDerivatives,
    ) {
        for (d, &b) in basis.iter().enumerate().take(MAX + 1) {
            match weight {
                None => self.n[d] += pole * b,
                Some(w) => {
                    self.n[d] += pole * (w * b);
                    self.w[d] += w * b;
                }
            }
        }
    }
}
