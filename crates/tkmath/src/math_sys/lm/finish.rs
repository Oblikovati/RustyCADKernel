// Copyright (c) 2026 OPEN CASCADE SAS
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

//! MathSys_LevenbergMarquardt.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{LMState, gradient, owned};
use crate::{
    math::{FunctionSetWithDerivatives, Matrix, Vector},
    math_lin::utils::{is_finite_matrix, is_finite_vector},
    math_sys::{
        SystemResult,
        linear::{safe_norm, scaled_rank, zero_matrix},
    },
    math_utils::Status,
};
impl LMState {
    pub(super) fn finish(
        mut self,
        f: &mut impl FunctionSetWithDerivatives,
        exhausted: bool,
    ) -> SystemResult {
        if self.result.status == Status::NotConverged && exhausted {
            self.result.status = Status::MaxIterations;
        }
        if matches!(
            self.result.status,
            Status::CallbackError | Status::NumericalError
        ) {
            self.result.solution = Some(self.solution);
            return self.result;
        }
        match self.finalize(f) {
            Err(status) => self.result.status = status,
            Ok(()) if self.result.is_root || self.result.is_stationary => {
                self.result.status = Status::Ok
            }
            Ok(()) if self.result.status == Status::Ok => self.result.status = Status::NotConverged,
            Ok(()) => {}
        }
        self.result
    }
    fn finalize(&mut self, f: &mut impl FunctionSetWithDerivatives) -> Result<(), Status> {
        let m: usize = self.residual.length() as usize;
        let n: usize = self.solution.length() as usize;
        let mut residual: Vector<'static> = Vector::with_size(m);
        let mut jacobian: Matrix<'static> = zero_matrix(m, n);
        self.result.solution = Some(owned(&self.solution));
        if !f.value(&self.solution, &mut residual) {
            return Err(Status::CallbackError);
        }
        if !is_finite_vector(&residual) {
            return Err(Status::NumericalError);
        }
        if !f.derivatives(&self.solution, &mut jacobian) {
            return Err(Status::CallbackError);
        }
        if !is_finite_matrix(&jacobian) {
            return Err(Status::NumericalError);
        }
        let rank: usize =
            scaled_rank(&jacobian, self.config.rank_tolerance).ok_or(Status::NumericalError)?;
        self.finish_fields(residual, jacobian, rank)
    }
    fn finish_fields(
        &mut self,
        residual: Vector<'static>,
        jacobian: Matrix<'static>,
        rank: usize,
    ) -> Result<(), Status> {
        let gradient: Vector<'static> = gradient(&jacobian, &residual);
        self.result.residual_norm = safe_norm(&residual);
        let square: f64 = self.result.residual_norm * self.result.residual_norm;
        if square.is_finite() {
            self.result.value = Some(square);
        }
        let finite: bool = is_finite_vector(&gradient);
        let norm: f64 = if finite {
            self.projected_norm(&gradient)
        } else {
            0.0
        };
        self.result.gradient = Some(gradient);
        if !finite {
            return Err(Status::NumericalError);
        }
        self.result.jacobian = Some(jacobian);
        self.result.rank = rank;
        self.result.is_root = self.result.residual_norm
            <= self.config.solver.f_tolerance
                + self.config.solver.relative_tolerance * self.initial_norm;
        self.result.is_stationary = norm <= self.config.solver.tolerance;
        Ok(())
    }
}
