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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_AHTBezierSurface.cxx (cubicVal, cubicD1..cubicD3, evalPolynomialSurfaceCubicNonRational, evalPolynomialSurfaceNonRational) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The purely algebraic, non-rational AHT-Bezier surface: nested Horner schemes
//! (a dedicated bicubic form when both degrees are 3), with the derivatives up to
//! the evaluator order, in `SurfaceDerivs` order (S00, S10, S01, S20, S02, S11, S30,
//! S03, S21, S12).
use crate::geom_eval::bezier_grid::PoleGrid;
use crate::geom_eval::bezier_results::SurfaceDerivs;
use rustycad_tkmath::gp::Xyz;

/// The sums of the evaluator of order `max` (0..=3): the bicubic form when both
/// degrees are 3, the general Horner scheme otherwise; `polynomial(&grid, [2, 2], 0.5, 0.5, 1)`.
pub(super) fn polynomial(
    grid: &PoleGrid<'_>,
    degrees: [usize; 2],
    u: f64,
    v: f64,
    max: usize,
) -> SurfaceDerivs {
    if degrees == [3, 3] {
        return bicubic(grid, u, v, max);
    }
    horner(grid, degrees, u, v, max)
}

// `((c3 t + c2) t + c1) t + c0`.
fn cubic_val(c: [Xyz; 4], t: f64) -> Xyz {
    ((c[3] * t + c[2]) * t + c[1]) * t + c[0]
}
fn cubic_d1(c: [Xyz; 4], t: f64) -> Xyz {
    (c[3] * (3.0 * t) + c[2] * 2.0) * t + c[1]
}
fn cubic_d2(c: [Xyz; 4], t: f64) -> Xyz {
    c[3] * (6.0 * t) + c[2] * 2.0
}
fn cubic_d3(c: [Xyz; 4]) -> Xyz {
    c[3] * 6.0
}

// `evalPolynomialSurfaceCubicNonRational<max>`: the U cubics of each column, then the V
// cubic of those coefficients.
fn bicubic(grid: &PoleGrid<'_>, u: f64, v: f64, max: usize) -> SurfaceDerivs {
    let column = |j: usize| -> [Xyz; 4] { std::array::from_fn(|i| grid.pole(i, j)) };
    let columns: [[Xyz; 4]; 4] = std::array::from_fn(column);
    let q0: [Xyz; 4] = columns.map(|c| cubic_val(c, u));
    let mut s: SurfaceDerivs = [Xyz::default(); 10];
    s[0] = cubic_val(q0, v);
    if max < 1 {
        return s;
    }
    let q1: [Xyz; 4] = columns.map(|c| cubic_d1(c, u));
    (s[1], s[2]) = (cubic_val(q1, v), cubic_d1(q0, v));
    if max < 2 {
        return s;
    }
    let q2: [Xyz; 4] = columns.map(|c| cubic_d2(c, u));
    (s[3], s[5], s[4]) = (cubic_val(q2, v), cubic_d1(q1, v), cubic_d2(q0, v));
    if max < 3 {
        return s;
    }
    let q3: [Xyz; 4] = columns.map(cubic_d3);
    (s[6], s[8], s[9], s[7]) = (
        cubic_val(q3, v),
        cubic_d1(q2, v),
        cubic_d2(q1, v),
        cubic_d3(q0),
    );
    s
}

// The U derivatives `D0..D3` of one column at `u`, by Horner from the top degree.
fn column_derivatives(grid: &PoleGrid<'_>, degree_u: usize, j: usize, u: f64, max: usize) -> [Xyz; 4] {
    let mut d: [Xyz; 4] = [grid.pole(degree_u, j), Xyz::default(), Xyz::default(), Xyz::default()];
    for i in (0..degree_u).rev() {
        if max >= 3 {
            d[3] = d[3] * u + d[2] * 3.0;
        }
        if max >= 2 {
            d[2] = d[2] * u + d[1] * 2.0;
        }
        if max >= 1 {
            d[1] = d[1] * u + d[0];
        }
        d[0] = d[0] * u + grid.pole(i, j);
    }
    d
}

// The running V accumulators `R(du, dv)` of the native scheme.
#[derive(Default)]
struct HornerRows {
    r00: Xyz,
    r01: Xyz,
    r02: Xyz,
    r03: Xyz,
    r10: Xyz,
    r11: Xyz,
    r12: Xyz,
    r20: Xyz,
    r21: Xyz,
    r30: Xyz,
}
impl HornerRows {
    // One V step with the column derivatives `d`, in the native statement order.
    fn step(&mut self, d: [Xyz; 4], v: f64, max: usize) {
        if max >= 3 {
            self.r03 = self.r03 * v + self.r02 * 3.0;
        }
        if max >= 2 {
            self.r02 = self.r02 * v + self.r01 * 2.0;
        }
        if max >= 1 {
            self.r01 = self.r01 * v + self.r00;
        }
        self.r00 = self.r00 * v + d[0];
        if max >= 1 {
            if max >= 2 {
                self.r12 = self.r12 * v + self.r11 * 2.0;
            }
            self.r11 = self.r11 * v + self.r10;
            self.r10 = self.r10 * v + d[1];
        }
        if max >= 2 {
            if max >= 3 {
                self.r21 = self.r21 * v + self.r20;
            }
            self.r20 = self.r20 * v + d[2];
        }
        if max >= 3 {
            self.r30 = self.r30 * v + d[3];
        }
    }
}

// `evalPolynomialSurfaceNonRational<max>`: columns from the top V degree down.
fn horner(grid: &PoleGrid<'_>, [degree_u, degree_v]: [usize; 2], u: f64, v: f64, max: usize) -> SurfaceDerivs {
    let mut rows = HornerRows::default();
    for j in (0..=degree_v).rev() {
        let d: [Xyz; 4] = column_derivatives(grid, degree_u, j, u, max);
        rows.step(d, v, max);
    }
    let mut s: SurfaceDerivs = [Xyz::default(); 10];
    s[0] = rows.r00;
    if max >= 1 {
        (s[2], s[1]) = (rows.r01, rows.r10);
    }
    if max >= 2 {
        (s[4], s[5], s[3]) = (rows.r02, rows.r11, rows.r20);
    }
    if max >= 3 {
        (s[7], s[9], s[8], s[6]) = (rows.r03, rows.r12, rows.r21, rows.r30);
    }
    s
}
