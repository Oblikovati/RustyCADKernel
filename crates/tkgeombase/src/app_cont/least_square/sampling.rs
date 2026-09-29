// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AppCont/AppCont_LeastSquare.cxx (sampling)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sampling part of the `AppCont_LeastSquare` constructor: the Gauss parameters
//! and weights, the function points at them, and the fix of period jumps between
//! consecutive samples.
use super::{Evaluation, LeastSquare};
use crate::app_cont::cont_matrices::THE_MAX_NBPOINTS;
use crate::app_cont::function::Function;
use crate::native_failure::failure;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Vector, gauss_points, gauss_points_max, gauss_weights};

impl LeastSquare {
    // The Gauss parameters (`myParam`) and the weights, then the points of the
    // function there (`myPoints`); returns the weights (`TheWeights`).
    pub(super) fn sample(
        &mut self,
        ssp: &dyn Function,
        u: [f64; 2],
        arrays: &mut Evaluation,
    ) -> Result<Vector<'static>, Failure> {
        let weights: Vector<'static> = self.gauss_parameters(u)?;
        for i in 1..=self.my_nbdiscret {
            ssp.value(self.my_param[i], &mut arrays.tab_p2d, &mut arrays.tab_p);
            self.store_point(i, arrays);
        }
        Ok(weights)
    }

    // The Gauss points mapped to `[U0, U1]` in increasing order, with half weights.
    // `VBParam` (the points mapped to [0, 1]) is computed natively and never read, so
    // it is not kept.
    fn gauss_parameters(&mut self, [u0, u1]: [f64; 2]) -> Result<Vector<'static>, Failure> {
        let n: i32 = self.my_nbdiscret;
        let (gauss_p, gauss_w) = gauss_tables(n)?;
        let mut the_weights: Vector<'static> = Vector::with_bounds(1, n)?;
        let du: f64 = 0.5 * (u1 - u0);
        for i in 1..=n {
            let u: f64 = 0.5 * (u1 + u0) + du * gauss_p[i];
            let slot: i32 = if i <= (n + 1) / 2 {
                n - i + 1
            } else {
                i - (n + 1) / 2
            };
            self.my_param[slot] = u;
            the_weights[slot] = 0.5 * gauss_w[i];
        }
        Ok(the_weights)
    }

    // `myPoints(i, .)`: the coordinates of the 3D then the 2D points.
    fn store_point(&mut self, i: i32, arrays: &Evaluation) {
        let mut i2: i32 = 1;
        for j in 1..=self.my_nb_p {
            let point = arrays.tab_p[j];
            [point.x(), point.y(), point.z()]
                .into_iter()
                .zip(i2..)
                .for_each(|(coordinate, k)| self.my_points[(i, k)] = coordinate);
            i2 += 3;
        }
        for j in 1..=self.my_nb_p2d {
            let point = arrays.tab_p2d[j];
            self.my_points[(i, i2)] = point.x();
            self.my_points[(i, i2 + 1)] = point.y();
            i2 += 2;
        }
    }

    // `myPoints(row, dim)`; a row past the samples (read natively, undefined
    // behavior, when fewer than 3 samples are periodic) is Standard_OutOfRange.
    fn point(&self, row: i32, dim: i32) -> Result<f64, Failure> {
        self.my_points.value(row, dim).copied()
    }

    // Brings the first sample next to the second when only the first step jumps a
    // period, then brings each sample next to its predecessor.
    pub(super) fn fix_period_jumps(&mut self) -> Result<(), Failure> {
        let max_dim: i32 = 3 * self.my_nb_p + 2 * self.my_nb_p2d;
        for dim_idx in 1..=max_dim {
            self.fix_first_sample(dim_idx)?;
        }
        for pnt_idx in 1..self.my_nbdiscret {
            for dim_idx in 1..=max_dim {
                self.fix_next_sample(pnt_idx, dim_idx)?;
            }
        }
        Ok(())
    }

    // The negated tests keep the native NaN behavior (no fix).
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn fix_first_sample(&mut self, dim_idx: i32) -> Result<(), Failure> {
        let info = self.per_info(dim_idx)?;
        let half: f64 = info.my_period / 2.01;
        if !(info.is_periodic && (self.point(1, dim_idx)? - self.point(2, dim_idx)?).abs() > half) {
            return Ok(());
        }
        if !((self.point(2, dim_idx)? - self.point(3, dim_idx)?).abs() < half) {
            return Ok(());
        }
        let first: f64 = self.point(1, dim_idx)?;
        let period_mult: f64 = if first < self.point(2, dim_idx)? {
            1.0
        } else {
            -1.0
        };
        self.my_points[(1, dim_idx)] = first + period_mult * info.my_period;
        Ok(())
    }

    // The negated tests keep the native NaN behavior (no fix).
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn fix_next_sample(&mut self, pnt_idx: i32, dim_idx: i32) -> Result<(), Failure> {
        let info = self.per_info(dim_idx)?;
        if !info.is_periodic {
            return Ok(());
        }
        let (current, next) = (
            self.point(pnt_idx, dim_idx)?,
            self.point(pnt_idx + 1, dim_idx)?,
        );
        if !((current - next).abs() > info.my_period / 2.01) {
            return Ok(());
        }
        let period_mult: f64 = if current > next { 1.0 } else { -1.0 };
        self.my_points[(pnt_idx + 1, dim_idx)] = next + period_mult * info.my_period;
        Ok(())
    }
}

// `math::GaussPoints` and `math::GaussWeights` of order `n`. Past the 61 tabulated
// orders the native functions read beyond their tables (undefined behavior) before
// VBernstein rejects `n > 24`; the port raises that Standard_DimensionError first.
fn gauss_tables(n: i32) -> Result<(Vector<'static>, Vector<'static>), Failure> {
    if n > gauss_points_max() {
        return Err(failure(
            "Standard_DimensionError",
            &format!("VBernstein: nbpoints = {n}; expected nbpoints <= {THE_MAX_NBPOINTS}"),
        ));
    }
    let mut gauss_p: Vector<'static> = Vector::with_bounds(1, n)?;
    let mut gauss_w: Vector<'static> = Vector::with_bounds(1, n)?;
    gauss_points(n, &mut gauss_p);
    gauss_weights(n, &mut gauss_w);
    Ok((gauss_p, gauss_w))
}
