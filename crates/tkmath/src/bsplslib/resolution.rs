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
// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib.cxx (Resolution) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Parametric tolerances from bounds on the partial derivatives of a surface.
//!
//! As for curves, |dS/du| <= UDegree * max |S(i,j) - S(i-1,j)| / (t(i+d) - t(i)),
//! with weighted differences over neighbouring poles in the rational case, divided
//! by the smallest weight; both bounds are multiplied by sqrt(2). The native loop
//! over neighbouring poles for V swaps the index ranges (see `rational`).
use crate::bsplclib::{knot_sequence_length, knot_sequence_with_degree};
use crate::gp::Point3;
use crate::plib::access::{array_value, grid_value};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// Knots, multiplicities, degree, rationality and periodicity of one direction.
#[derive(Clone, Copy)]
pub struct ResolutionDirection<'r, 'a> {
    /// Knots.
    pub knots: &'r Array1<'a, f64>,
    /// Multiplicities.
    pub mults: &'r Array1<'a, i32>,
    /// Degree.
    pub degree: i32,
    /// Rationality.
    pub rational: bool,
    /// Periodicity.
    pub periodic: bool,
}

/// U and V tolerances matching `tolerance_3d` (`BSplSLib::Resolution`); both are 0
/// when a derivative bound is 0; `bsplslib::resolution(&poles, None, u, v, 1e-7)?`.
pub fn resolution(
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
    u: ResolutionDirection<'_, '_>,
    v: ResolutionDirection<'_, '_>,
    tolerance_3d: f64,
) -> Result<(f64, f64), Failure> {
    let u_flat: Array1<'static, f64> = flat_knots(&u)?;
    let v_flat: Array1<'static, f64> = flat_knots(&v)?;
    let min_weights: f64 = if u.rational || v.rational {
        smallest_weight(weights, poles)?
    } else {
        0.0
    };
    let bounds = Bounds {
        poles,
        weights,
        num_poles: [
            u_flat.len() as i32 - (u.degree + 1),
            v_flat.len() as i32 - (v.degree + 1),
        ],
        lengths: [poles.nb_rows() as i32, poles.nb_columns() as i32],
        degrees: [u.degree, v.degree],
    };
    let mut max_u: f64 = if u.rational {
        bounds.rational(0, &u_flat)? / min_weights
    } else {
        bounds.polynomial(0, &u_flat)?
    };
    max_u *= f64::from(u.degree);
    let mut max_v: f64 = if v.rational {
        bounds.rational(1, &v_flat)? / min_weights
    } else {
        bounds.polynomial(1, &v_flat)?
    };
    max_v *= f64::from(v.degree);
    max_u *= std::f64::consts::SQRT_2;
    max_v *= std::f64::consts::SQRT_2;
    // Non-zero tests, as the native `if (max_derivative[0] && max_derivative[1])`.
    if max_u != 0.0 && max_v != 0.0 {
        return Ok((tolerance_3d / max_u, tolerance_3d / max_v));
    }
    Ok((0.0, 0.0))
}

fn flat_knots(direction: &ResolutionDirection<'_, '_>) -> Result<Array1<'static, f64>, Failure> {
    let length: i32 = knot_sequence_length(direction.mults, direction.degree, direction.periodic)?;
    let mut flat: Array1<'static, f64> = Array1::with_bounds(1, length)
        .map_err(|error| Failure::out_of_range(&format!("BSplSLib::Resolution: {error}")))?;
    knot_sequence_with_degree(
        direction.knots,
        direction.mults,
        direction.degree,
        direction.periodic,
        &mut flat,
    )?;
    Ok(flat)
}

// Minimum over the RowLength*ColLength weights read from Weights(LowerRow, LowerCol).
fn smallest_weight(
    weights: Option<&Array2<'_, f64>>,
    poles: &Array2<'_, Point3>,
) -> Result<f64, Failure> {
    let weights = weights.ok_or_else(|| {
        Failure::out_of_range("BSplSLib::Resolution: rational surface needs Weights; received None")
    })?;
    let values: &[f64] = weights.as_slice();
    let size: usize = poles.len();
    if values.len() < size.max(1) {
        return Err(Failure::out_of_range(&format!(
            "BSplSLib::Resolution: {} weights for {size} poles",
            values.len()
        )));
    }
    let mut min_weights: f64 = values[0];
    for &w in &values[1..size] {
        if min_weights > w {
            min_weights = w;
        }
    }
    Ok(min_weights)
}

struct Bounds<'r, 'a, 'w> {
    poles: &'r Array2<'a, Point3>,
    weights: Option<&'r Array2<'w, f64>>,
    // [U, V] pole counts from the flat knots and grid lengths (rows, columns).
    num_poles: [i32; 2],
    lengths: [i32; 2],
    degrees: [i32; 2],
}

// `if (factor < 0) factor = -factor;`, which keeps -0.0.
fn magnitude(factor: f64) -> f64 {
    if factor < 0.0 { -factor } else { factor }
}

impl Bounds<'_, '_, '_> {
    // Grid cell with `along` in direction `dir` and `across` in the other one.
    fn cell(&self, dir: usize, along: i32, across: i32) -> (i32, i32) {
        if dir == 0 {
            (along, across)
        } else {
            (across, along)
        }
    }

    fn cyclic(&self, dir: usize, index: i32) -> i32 {
        (index - 1) % self.lengths[dir] + 1
    }

    fn inverse(&self, dir: usize, flat: &Array1<'_, f64>, ii: i32) -> Result<f64, Failure> {
        let span: f64 = array_value(flat, ii + self.degrees[dir])? - array_value(flat, ii)?;
        Ok(1.0e0 / span)
    }

    fn polynomial(&self, dir: usize, flat: &Array1<'_, f64>) -> Result<f64, Failure> {
        let other: usize = 1 - dir;
        let mut max_derivative: f64 = 0.0e0;
        for ii in 2..=self.num_poles[dir] {
            let (index, minus) = (self.cyclic(dir, ii), self.cyclic(dir, ii - 1));
            let inverse: f64 = self.inverse(dir, flat, ii)?;
            for jj in 1..=self.num_poles[other] {
                let across: i32 = self.cyclic(other, jj);
                let (r1, c1) = self.cell(dir, index, across);
                let (r2, c2) = self.cell(dir, minus, across);
                let (p, m): (Point3, Point3) = (
                    grid_value(self.poles, r1, c1)?,
                    grid_value(self.poles, r2, c2)?,
                );
                let mut value: f64 = 0.0e0;
                value += magnitude(p.x() - m.x());
                value += magnitude(p.y() - m.y());
                value += magnitude(p.z() - m.z());
                value *= inverse;
                if max_derivative < value {
                    max_derivative = value;
                }
            }
        }
        Ok(max_derivative)
    }

    fn rational(&self, dir: usize, flat: &Array1<'_, f64>) -> Result<f64, Failure> {
        let other: usize = 1 - dir;
        let weights = self.weights.ok_or_else(|| {
            Failure::out_of_range(
                "BSplSLib::Resolution: rational surface needs Weights; received None",
            )
        })?;
        let (d1, d_other) = (self.degrees[dir] + 1, self.degrees[other] + 1);
        let mut max_derivative: f64 = 0.0e0;
        for ii in 2..=self.num_poles[dir] {
            let (index, minus) = (self.cyclic(dir, ii), self.cyclic(dir, ii - 1));
            let inverse: f64 = self.inverse(dir, flat, ii)?;
            let lower0: i32 = (ii - d1).max(1);
            let upper0: i32 = (ii + (self.degrees[dir] << 1) + 1).min(self.num_poles[dir]);
            for jj in 1..=self.num_poles[other] {
                let across: i32 = self.cyclic(other, jj);
                let lower1: i32 = (jj - d_other).max(1);
                let upper1: i32 = (jj + (self.degrees[other] << 1) + 1).min(self.num_poles[other]);
                let (r1, c1) = self.cell(dir, index, across);
                let (r2, c2) = self.cell(dir, minus, across);
                let pij: Point3 = grid_value(self.poles, r1, c1)?;
                let wij: f64 = grid_value(weights, r1, c1)?;
                let pmj: Point3 = grid_value(self.poles, r2, c2)?;
                let wmj: f64 = grid_value(weights, r2, c2)?;
                let terms = RationalTerms {
                    pij,
                    wij,
                    pmj,
                    wmj,
                    inverse,
                };
                for pp in lower0..=upper0 {
                    for qq in lower1..=upper1 {
                        // Quirk kept: in both directions the neighbour row comes from the
                        // range of the scanned direction and the column from the other one,
                        // so along V the U range indexes columns and the V range rows.
                        let (r, c) = (self.cyclic(0, pp), self.cyclic(1, qq));
                        let value: f64 = terms.value(grid_value(self.poles, r, c)?);
                        if max_derivative < value {
                            max_derivative = value;
                        }
                    }
                }
            }
        }
        Ok(max_derivative)
    }
}

struct RationalTerms {
    pij: Point3,
    wij: f64,
    pmj: Point3,
    wmj: f64,
    inverse: f64,
}

impl RationalTerms {
    // Sum over x, y, z of |(Ppq - Pij) Wij - (Ppq - Pmj) Wmj|, times the inverse span.
    fn value(&self, ppq: Point3) -> f64 {
        let mut value: f64 = 0.0e0;
        for k in 0..3 {
            let q: f64 = ppq.coordinates()[k];
            let mut factor: f64 = (q - self.pij.coordinates()[k]) * self.wij;
            factor -= (q - self.pmj.coordinates()[k]) * self.wmj;
            value += magnitude(factor);
        }
        value * self.inverse
    }
}
