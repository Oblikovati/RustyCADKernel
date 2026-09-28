// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/gp/gp_Torus.cxx Coefficients at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The 35 quartic coefficients of gp_Torus::Coefficients, grouped by the product shapes the
//! native code repeats; every group keeps the native multiplication and summation order.
use super::super::placement::column_product;

/// Column dot products `c[i][j] = Σ T(r,i)·T(r,j)` over the three frame-change rows.
struct Columns {
    all: [[f64; 4]; 4],
    planar: [[f64; 4]; 4],
    rows: [[f64; 4]; 3],
    sum: f64,
    difference: f64,
}
impl Columns {
    fn new(rows: [[f64; 4]; 3], major: f64, minor: f64) -> Self {
        Self {
            all: std::array::from_fn(|i| std::array::from_fn(|j| column_product(&rows, i, j))),
            planar: std::array::from_fn(|i| {
                std::array::from_fn(|j| column_product(&rows[..2], i, j))
            }),
            rows,
            sum: major * major + minor * minor,
            difference: major * major - minor * minor,
        }
    }
    /// `4·c[a][a]·c[a][b]` (entries 3-8 and 15-17).
    fn linear(&self, a: usize, b: usize) -> f64 {
        4.0 * self.all[a][a] * self.all[a][b]
    }
    /// `c[a][a]·c[b][c] + 2·c[a][b]·c[a][c]` (entries 9-14 and 18-23 before scaling).
    fn mixed(&self, a: usize, b: usize, c: usize) -> f64 {
        self.all[a][a] * self.all[b][c] + 2.0 * self.all[a][b] * self.all[a][c]
    }
    /// Quadratic radius terms of entries 25-30 before scaling.
    fn radial(&self, i: usize, j: usize) -> f64 {
        self.difference * self.rows[2][i] * self.rows[2][j] - self.sum * self.planar[i][j]
            + self.all[3][3] * self.all[i][j]
            + 2.0 * self.all[i][3] * self.all[j][3]
    }
    /// Linear radius terms of entries 31-33 before scaling.
    fn axial(&self, i: usize) -> f64 {
        self.all[3][3] * self.all[i][3] + self.difference * self.rows[2][i] * self.rows[2][3]
            - self.sum * self.planar[i][3]
    }
    fn constant(&self) -> f64 {
        let t34 = self.rows[2][3];
        2.0 * self.difference * t34 * t34 - 2.0 * self.sum * self.planar[3][3]
            + self.all[3][3] * self.all[3][3]
            + self.difference * self.difference
    }
    fn cross_terms(&self) -> f64 {
        let c = &self.all;
        8.0 * (c[0][1] * c[2][3] + c[1][2] * c[0][3] + c[1][3] * c[0][2])
    }
}

/// Evaluates gp_Torus::Coefficients for the frame-change rows; `quartic(rows,5.0,1.0)`.
pub(super) fn quartic(rows: [[f64; 4]; 3], major: f64, minor: f64) -> [f64; 35] {
    let c = Columns::new(rows, major, minor);
    let squares = [0, 1, 2].map(|a| c.all[a][a] * c.all[a][a]);
    let linear = [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)].map(|(a, b)| c.linear(a, b));
    let pairs = [(0, 1), (0, 2), (1, 2)].map(|(a, b)| 2.0 * c.mixed(a, b, b));
    let triples = [(0, 1, 2), (1, 0, 2), (2, 0, 1)].map(|(a, b, d)| 4.0 * c.mixed(a, b, d));
    let offsets = [0, 1, 2].map(|a| c.linear(a, 3));
    let offset_mixed =
        [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)].map(|(a, d)| 4.0 * c.mixed(a, 3, d));
    let radial_squares = [0, 1, 2].map(|i| 2.0 * c.radial(i, i));
    let radial_cross = [(0, 1), (0, 2), (1, 2)].map(|(i, j)| 4.0 * c.radial(i, j));
    let axial = [0, 1, 2].map(|i| 4.0 * c.axial(i));
    let mut values = [0.0; 35];
    let groups: [&[f64]; 11] = [
        &squares,
        &linear,
        &pairs,
        &triples,
        &offsets,
        &offset_mixed,
        &[c.cross_terms()],
        &radial_squares,
        &radial_cross,
        &axial,
        &[c.constant()],
    ];
    let mut index = 0;
    for group in groups {
        values[index..index + group.len()].copy_from_slice(group);
        index += group.len();
    }
    values
}
