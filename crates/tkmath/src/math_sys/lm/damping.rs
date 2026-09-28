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
use super::{LMState, merit, owned};
use crate::{
    math::{FunctionSetWithDerivatives, Matrix, Vector},
    math_lin::utils::is_finite_vector,
    math_poly::extended::LongDouble as E,
    math_sys::linear::{safe_norm, solve_linearized, zero_matrix},
    math_utils::Status,
};
impl LMState {
    pub(super) fn damping(
        &mut self,
        f: &mut impl FunctionSetWithDerivatives,
        gradient: &Vector<'_>,
    ) -> bool {
        for _ in 0..super::LM_DAMPING_MAX {
            let (matrix, rhs): (Matrix<'static>, Vector<'static>) = self.augmented();
            let step = solve_linearized(&matrix, &rhs, self.config.rank_tolerance);
            let Some(step) = step.solution else {
                if !self.increase(self.config.lambda_increase) {
                    break;
                }
                continue;
            };
            let trial: Vector<'static> = self.trial(&step);
            if !self.evaluate_trial(f, &trial) {
                break;
            }
            let trial_merit: E = merit(safe_norm(&self.trial_residual));
            let predicted: E = self.predicted(&trial, gradient);
            let rho: E = if predicted > E::ZERO {
                (self.merit - trial_merit) / predicted
            } else {
                E::new(-1.0)
            };
            if rho > E::ZERO && trial_merit < self.merit {
                self.accept(trial, trial_merit, rho);
                return true;
            }
            if !self.increase(if rho < E::new(0.25) {
                self.config.lambda_increase
            } else {
                2.0
            }) {
                break;
            }
        }
        false
    }
    fn augmented(&self) -> (Matrix<'static>, Vector<'static>) {
        let m: usize = self.residual.length() as usize;
        let n: usize = self.solution.length() as usize;
        let mut matrix: Matrix<'static> = zero_matrix(m + n, n);
        let mut rhs: Vector<'static> = Vector::with_size(m + n);
        for row in 0..m {
            *rhs.change_at(row).unwrap() = -self.residual.at(row).unwrap();
            for col in 0..n {
                *matrix.change_at(row, col).unwrap() = *self.jacobian.at(row, col).unwrap();
            }
        }
        for col in 0..n {
            let norm: f64 = (0..m).fold(0.0_f64, |norm, row| {
                norm.hypot(*self.jacobian.at(row, col).unwrap())
            });
            *matrix.change_at(m + col, col).unwrap() = self.lambda.sqrt() * norm.max(1.0);
        }
        (matrix, rhs)
    }
    fn trial(&self, step: &Vector<'_>) -> Vector<'static> {
        let mut trial: Vector<'static> = Vector::with_size(self.solution.length() as usize);
        for i in 0..self.solution.length() as usize {
            *trial.change_at(i).unwrap() =
                self.clamp(i, self.solution.at(i).unwrap() + step.at(i).unwrap());
        }
        trial
    }
    fn evaluate_trial(
        &mut self,
        f: &mut impl FunctionSetWithDerivatives,
        trial: &Vector<'_>,
    ) -> bool {
        if !is_finite_vector(trial) {
            self.result.status = Status::NumericalError;
            return false;
        }
        if !f.value(trial, &mut self.trial_residual) {
            self.result.status = Status::CallbackError;
            return false;
        }
        if !is_finite_vector(&self.trial_residual) {
            self.result.status = Status::NumericalError;
            return false;
        }
        true
    }
    fn predicted(&self, trial: &Vector<'_>, gradient: &Vector<'_>) -> E {
        let mut predicted: E = E::ZERO;
        for col in 0..self.solution.length() as usize {
            let step: f64 = trial.at(col).unwrap() - self.solution.at(col).unwrap();
            predicted = predicted - E::new(*gradient.at(col).unwrap()) * E::new(step);
        }
        let mut norm: f64 = 0.0;
        for row in 0..self.residual.length() as usize {
            let mut value: f64 = 0.0;
            for col in 0..self.solution.length() as usize {
                value += self.jacobian.at(row, col).unwrap()
                    * (trial.at(col).unwrap() - self.solution.at(col).unwrap());
            }
            norm = norm.hypot(value);
        }
        predicted - merit(norm)
    }
    fn accept(&mut self, trial: Vector<'static>, trial_merit: E, rho: E) {
        self.result.step_norm = 0.0;
        for col in 0..self.solution.length() as usize {
            self.result.step_norm = self
                .result
                .step_norm
                .hypot(trial.at(col).unwrap() - self.solution.at(col).unwrap());
        }
        self.solution = trial;
        self.residual = owned(&self.trial_residual);
        self.merit = trial_merit;
        self.lambda = self.config.lambda_min.max(
            self.lambda
                * if rho > E::new(0.75) {
                    self.config.lambda_decrease
                } else {
                    1.0
                },
        );
    }
    fn increase(&mut self, factor: f64) -> bool {
        if self.lambda >= self.config.lambda_max {
            return false;
        }
        self.lambda = self.config.lambda_max.min(self.lambda * factor);
        true
    }
}
