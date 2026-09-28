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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_LeastSquare.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two linear solves of `AppParCurves_LeastSquare::Perform`: the per-column
//! solve of the pass-point unknowns, and the full system with the tangency
//! lambdas, both through the profile Cholesky DACTCL.
use super::LeastSquare;
use crate::app_par_curves::constraint::Constraint;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::recipes::{dactcl_decompose, dactcl_solve};
use rustycad_tkmath::math::{IntegerVector, Matrix, Vector};

/// The native `MinPivot` default of DACTCL_Decompose and DACTCL_Solve.
const MIN_PIVOT: f64 = 1.0e-20;

impl LeastSquare {
    /// The unknown poles `resinit..=resfin`, one right-hand-side column at a time:
    /// `(tA A) X = tA B2` (the DACTCL statuses are ignored, as natively).
    pub(super) fn solve_blocks(&mut self) -> Result<(), Failure> {
        let nincx: i32 = self.resfin - self.resinit + 1;
        let columns: i32 = self.b2.col_number();
        let mut index: IntegerVector<'static> = IntegerVector::filled(1, nincx, 0)?;
        self.search_index(&mut index)?;
        let mut mytab: Matrix<'static> = Matrix::filled(self.resinit, self.resfin, 1, columns, 0.)
            .map_err(|error| {
                crate::native_failure::array_failure(error, "AppParCurves_LeastSquare")
            })?;
        let mut the_aa: Vector<'static> = Vector::filled(1, index[nincx], 0.)?;
        let mut tabb: Vector<'static> = Vector::filled(1, nincx, 0.)?;
        self.make_taa_blocks(&mut the_aa, &mut mytab)?;
        dactcl_decompose(&mut the_aa, &index, MIN_PIVOT)?;
        for j in 1..=columns {
            for (kk2, i) in (1..).zip(self.resinit..=self.resfin) {
                tabb[kk2] = mytab[(i, j)];
            }
            dactcl_solve(&the_aa, &mut tabb, &index, MIN_PIVOT)?;
            for (i2, k) in (1..).zip(self.resinit..=self.resfin) {
                *self.mypoles.change_value(k, j)? = tabb[i2];
            }
        }
        Ok(())
    }

    // The profile of the full system: the block profile repeated for each of the NA
    // coordinates, then the lambda columns.
    fn full_index(&mut self, nincx: i32, ninc1: i32) -> Result<IntegerVector<'static>, Failure> {
        let mut internal: IntegerVector<'static> = IntegerVector::filled(1, nincx.max(0), 0)?;
        self.search_index(&mut internal)?;
        let mut index: IntegerVector<'static> = IntegerVector::filled(1, self.ninc, 0)?;
        let mut l: i32 = 1;
        if self.resinit <= self.resfin {
            for j in 0..self.na {
                let deport: i32 = j * internal[nincx];
                for i in 1..=nincx {
                    *index.change_value(l)? = internal[i] + deport;
                    l += 1;
                }
            }
        }
        let (first, last) = (self.first_constraint, self.last_constraint);
        if self.resinit > self.resfin {
            *index.change_value(1)? = 1;
        }
        if ninc1 > 1 && first >= Constraint::TangencyPoint && last >= Constraint::TangencyPoint {
            *index.change_value(ninc1)? = *index.value(ninc1 - 1)? + ninc1;
        }
        if first >= Constraint::TangencyPoint || last >= Constraint::TangencyPoint {
            let ninc: i32 = self.ninc;
            *index.change_value(ninc)? = *index.value(ninc - 1)? + ninc;
        }
        Ok(index)
    }

    /// The "cas de tangence" part of `Perform(Parameters)`: every unknown pole of every
    /// coordinate and the lambdas in one symmetric system.
    pub(super) fn solve_full_system(&mut self) -> Result<(), Failure> {
        let ninc1: i32 = self.ninc - 1;
        let nincx: i32 = self.resfin - self.resinit + 1;
        let index: IntegerVector<'static> = self.full_index(nincx, ninc1)?;
        let mut the_a: Vector<'static> = Vector::filled(1, *index.value(self.ninc)?, 0.)?;
        let mut tab: Vector<'static> = Vector::filled(1, self.ninc, 0.)?;
        self.make_taa_full(&mut the_a, &mut tab)?;
        dactcl_decompose(&mut the_a, &index, MIN_PIVOT)?;
        if dactcl_solve(&the_a, &mut tab, &index, MIN_PIVOT)? == 0 {
            self.done = true;
        }
        self.store_lambdas(&tab, ninc1)?;
        self.store_poles(&tab, nincx)
    }

    fn store_lambdas(&mut self, tab: &Vector<'_>, ninc1: i32) -> Result<(), Failure> {
        let (first, last) = (
            self.first_constraint >= Constraint::TangencyPoint,
            self.last_constraint >= Constraint::TangencyPoint,
        );
        if first && last {
            self.lambda1 = *tab.value(ninc1)?;
            self.lambda2 = *tab.value(self.ninc)?;
        } else if first {
            self.lambda1 = *tab.value(self.ninc)?;
        } else if last {
            self.lambda2 = *tab.value(self.ninc)?;
        }
        Ok(())
    }

    // mypoles from the solution: the unknown rows of each coordinate block, then the
    // tangency poles 2 and NbPoles - 1 from the lambdas.
    fn store_poles(&mut self, tab: &Vector<'_>, nincx: i32) -> Result<(), Failure> {
        let (mut k, mut i2): (i32, i32) = (1, 1);
        let blocks: Vec<i32> = std::iter::repeat_n(3, self.nb_p as usize)
            .chain(std::iter::repeat_n(2, self.nb_p2d as usize))
            .collect();
        for width in blocks {
            for j in self.resinit..=self.resfin {
                for c in 0..width {
                    *self.mypoles.change_value(j, k + c)? = *tab.value(i2 + c * nincx)?;
                }
                i2 += 1;
            }
            self.tangency_poles(k, width)?;
            k += width;
            i2 += (width - 1) * nincx;
        }
        Ok(())
    }

    // mypoles(2, :) = P1 + lambda1 V1t and mypoles(NbPoles - 1, :) = Pn - lambda2 V2t
    // for the `width` columns from `k`.
    fn tangency_poles(&mut self, k: i32, width: i32) -> Result<(), Failure> {
        for c in k..k + width {
            if self.first_constraint >= Constraint::TangencyPoint {
                *self.mypoles.change_value(2, c)? = *self.mypoints.value(self.myfirstp, c)?
                    + self.lambda1 * *self.vec1t.value(c)?;
            }
        }
        for c in k..k + width {
            if self.last_constraint >= Constraint::TangencyPoint {
                *self.mypoles.change_value(self.nbpoles - 1, c)? =
                    *self.mypoints.value(self.mylastp, c)? - self.lambda2 * *self.vec2t.value(c)?;
            }
        }
        Ok(())
    }
}
