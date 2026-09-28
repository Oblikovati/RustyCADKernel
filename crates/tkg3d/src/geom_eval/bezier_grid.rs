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
// Rust translation of the pole-grid loops shared by ModelingData/TKG3d/GeomEval/GeomEval_TBezierSurface.cxx and GeomEval_AHTBezierSurface.cxx (the weight checks, evalTensorDerivs and the tensor sums of EvalD0..EvalDN) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pole grid of the GeomEval Bezier surfaces and the tensor-product sums
//! `N(du, dv) = sum_ij w_ij P_ij Bu_i^(du) Bv_j^(dv)` and `W(du, dv)` that both
//! surfaces accumulate, row by row (U major), as natively.
use super::eval_failures::unsupported;
use crate::geom::{dump_surface, errors};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::gp::{Point3, Xyz};
use std::io;

/// The poles and optional weights of a surface in storage order (row = U index),
/// as the native `Value(LowerRow() + i, LowerCol() + j)` reads them.
#[derive(Clone, Copy)]
pub(crate) struct PoleGrid<'a> {
    pub(crate) poles: &'a [Point3],
    pub(crate) weights: Option<&'a [f64]>,
    pub(crate) rows: usize,
    pub(crate) cols: usize,
}
impl PoleGrid<'_> {
    /// The coordinates of pole `(i, j)` (zero-based, U index first).
    pub(crate) fn pole(&self, i: usize, j: usize) -> Xyz {
        self.poles[i * self.cols + j].xyz()
    }
}

/// The derivatives of one axis basis, orders `0..=max`, stored order-major
/// (`theDerivs[d * theDim + i]`); `basis.at(1, 0)`.
pub(crate) struct AxisBasis {
    pub(crate) values: Vec<f64>,
    pub(crate) dim: usize,
}
impl AxisBasis {
    /// Zero-filled derivatives of orders `0..=max` for `dim` basis functions.
    pub(crate) fn zeros(max: usize, dim: usize) -> Self {
        Self {
            values: vec![0.0; (max + 1) * dim],
            dim,
        }
    }
    /// The order-`d` derivative of basis function `i`.
    pub(crate) fn at(&self, d: usize, i: usize) -> f64 {
        self.values[d * self.dim + i]
    }
    /// Mutable access to the order-`d` derivative of basis function `i`.
    pub(crate) fn at_mut(&mut self, d: usize, i: usize) -> &mut f64 {
        &mut self.values[d * self.dim + i]
    }
}

/// How the weight enters the rational product, which fixes the rounding:
/// the T-Bezier surface writes `w * Bu * Bv` (`(w * Bu) * Bv`), the AHT-Bezier
/// surface `w * (Bu * Bv)`.
#[derive(Clone, Copy)]
pub(crate) enum WeightProduct {
    WeightFirst,
    BasisFirst,
}
impl WeightProduct {
    fn apply(self, w: f64, bu: f64, bv: f64) -> f64 {
        match self {
            Self::WeightFirst => w * bu * bv,
            Self::BasisFirst => w * (bu * bv),
        }
    }
}

/// The `(du, dv)` pairs that EvalD0..EvalD3 accumulate, in result order:
/// S, DU, DV, D2U, D2V, D2UV, D3U, D3V, D3UUV, D3UVV.
pub(crate) const ORDER_PAIRS: [(usize, usize); 10] = [
    (0, 0),
    (1, 0),
    (0, 1),
    (2, 0),
    (0, 2),
    (1, 1),
    (3, 0),
    (0, 3),
    (2, 1),
    (1, 2),
];

/// The number of pairs of `ORDER_PAIRS` that an evaluator of order `max` uses.
pub(crate) fn pair_count(max: usize) -> usize {
    [1, 3, 6, 10][max]
}

/// The sums of each requested `(du, dv)` pair: `n[k]` and, for a rational grid, `w[k]`
/// (zero otherwise).
pub(crate) struct TensorSums {
    pub(crate) n: Vec<Xyz>,
    pub(crate) w: Vec<f64>,
}

/// Accumulate `N` and `W` for every pair of `orders`, looping over the poles row by
/// row; a non-rational grid adds `P * (Bu * Bv)`, a rational one `P * wB` and `wB`
/// with `wB` formed as `product` says; `tensor_sums(&grid, &bu, &bv, &[(0, 0)], p)`.
pub(crate) fn tensor_sums(
    grid: &PoleGrid<'_>,
    axes: [&AxisBasis; 2],
    orders: &[(usize, usize)],
    product: WeightProduct,
) -> TensorSums {
    let mut sums = TensorSums {
        n: vec![Xyz::default(); orders.len()],
        w: vec![0.0; orders.len()],
    };
    for i in 0..grid.rows {
        for j in 0..grid.cols {
            add_pole(&mut sums, grid, axes, orders, (i, j), product);
        }
    }
    sums
}

// One pole's contribution to every requested pair.
fn add_pole(
    sums: &mut TensorSums,
    grid: &PoleGrid<'_>,
    [bu, bv]: [&AxisBasis; 2],
    orders: &[(usize, usize)],
    (i, j): (usize, usize),
    product: WeightProduct,
) {
    let pole: Xyz = grid.pole(i, j);
    for (k, &(du, dv)) in orders.iter().enumerate() {
        let (basis_u, basis_v) = (bu.at(du, i), bv.at(dv, j));
        let Some(weights) = grid.weights else {
            sums.n[k] += pole * (basis_u * basis_v);
            continue;
        };
        let weighted: f64 = product.apply(weights[i * grid.cols + j], basis_u, basis_v);
        sums.n[k] += pole * weighted;
        sums.w[k] += weighted;
    }
}

/// The `(du, dv)` pairs of the full tensor `0..=max_u` x `0..=max_v`, du major, so that
/// the pair `(du, dv)` sits at the native `tensorIdx(du, dv, max_v)`.
pub(crate) fn full_tensor(max_u: usize, max_v: usize) -> Vec<(usize, usize)> {
    (0..=max_u)
        .flat_map(|du| (0..=max_v).map(move |dv| (du, dv)))
        .collect()
}

/// An owning copy that keeps the source bounds, as the NCollection_Array2 copy
/// constructor; `owned_grid(&poles)`.
pub(crate) fn owned_grid<T: Clone>(source: &Array2<'_, T>) -> Array2<'static, T> {
    let mut copy: Array2<'static, T> = Array2::default();
    copy.assign(source);
    copy
}

/// The native weight checks of the rational constructors, in order: the grid shape,
/// then every weight > 0 row by row; `check_weight_grid(CLASS, &weights, &poles)?`.
pub(crate) fn check_weight_grid(
    class: &str,
    weights: &Array2<'_, f64>,
    poles: &Array2<'_, Point3>,
) -> Result<(), Failure> {
    let (rows, cols) = (poles.nb_rows(), poles.nb_columns());
    if weights.nb_rows() != rows || weights.nb_columns() != cols {
        return Err(errors::construction(&format!(
            "{class}: weights are {}x{}; expected the pole grid shape {rows}x{cols}",
            weights.nb_rows(),
            weights.nb_columns()
        )));
    }
    let offset = |k: usize| (k / cols.max(1), k % cols.max(1));
    for (k, &weight) in weights.as_slice().iter().enumerate() {
        if weight <= 0.0 {
            let (i, j) = offset(k);
            return Err(errors::construction(&format!(
                "{class}: weight({}, {}) = {weight}; expected every weight > 0",
                weights.lower_row() + i as i32,
                weights.lower_col() + j as i32
            )));
        }
    }
    Ok(())
}

/// Standard_NotImplemented of a member the Bezier eval surfaces refuse (`UReverse`,
/// `UIso`, `Transform`...); `refuse(CLASS, "UIso", "(U = 0.5)")`.
pub(crate) fn refuse<T>(class: &str, member: &str, detail: &str) -> Result<T, Failure> {
    Err(unsupported(&format!("{class}::{member}"), detail))
}

/// The class name and the `OCCT_DUMP_BASE_CLASS(Geom_BoundedSurface)` level, which is the
/// inherited `Geom_Surface::DumpJson`; `dump_bounded_surface_base(&mut s, -1, CLASS)?`.
pub(crate) fn dump_bounded_surface_base(
    stream: &mut DumpStream,
    depth: i32,
    class: &str,
) -> io::Result<()> {
    dump_class_name(stream, class)?;
    dump_base(stream, depth, dump_surface)
}
