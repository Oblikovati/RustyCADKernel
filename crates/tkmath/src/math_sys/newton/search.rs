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

//! MathSys native behavior at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::NewtonState;
use crate::{
    math::{FunctionSetWithDerivatives, Vector},
    math_lin::utils::is_finite_vector,
    math_poly::extended::LongDouble as E,
    math_sys::linear::safe_norm,
    math_utils::Status,
};
impl NewtonState {
    pub(super) fn line_search(
        &mut self,
        f: &mut impl FunctionSetWithDerivatives,
        direction: &Vector<'_>,
        gradient: &Vector<'_>,
        norm: f64,
    ) -> Result<bool, Status> {
        let merit: E = E::new(0.5) * E::new(norm) * E::new(norm);
        let mut trial: Vector<'static> = Vector::with_size(self.solution.length() as usize);
        for iteration in 0..24 {
            let alpha: f64 = f64::from_bits((1023 - iteration as u64) << 52);
            let derivative: f64 = self.project_trial(&mut trial, direction, gradient, alpha);
            if derivative.partial_cmp(&0.0) != Some(std::cmp::Ordering::Less) {
                continue;
            }
            if !f.value(&trial, &mut self.trial_residual) {
                return Err(Status::CallbackError);
            }
            if !is_finite_vector(&self.trial_residual) {
                return Err(Status::NumericalError);
            }
            let trial_norm: f64 = safe_norm(&self.trial_residual);
            let trial_merit: E = E::new(0.5) * E::new(trial_norm) * E::new(trial_norm);
            if trial_merit <= merit + (E::new(1.0) / E::new(10000.0)) * E::new(derivative) {
                self.accept_trial(trial);
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn project_trial(
        &self,
        trial: &mut Vector<'_>,
        direction: &Vector<'_>,
        gradient: &Vector<'_>,
        alpha: f64,
    ) -> f64 {
        let mut derivative: f64 = 0.0;
        for i in 0..self.solution.length() as usize {
            let mut value: f64 = self.solution.at(i).unwrap() + alpha * direction.at(i).unwrap();
            if let Some((lower, upper)) = &self.bounds {
                value = value.clamp(*lower.at(i).unwrap(), *upper.at(i).unwrap());
            }
            *trial.change_at(i).unwrap() = value;
            derivative += gradient.at(i).unwrap() * (value - self.solution.at(i).unwrap());
        }
        derivative
    }
    fn accept_trial(&mut self, trial: Vector<'static>) {
        self.result.step_norm = 0.0;
        self.within_x = true;
        for i in 0..self.solution.length() as usize {
            let step: f64 = trial.at(i).unwrap() - self.solution.at(i).unwrap();
            self.result.step_norm = self.result.step_norm.hypot(step);
            self.within_x = self.within_x && step.abs() <= *self.tolerance.at(i).unwrap();
        }
        self.solution = trial;
        self.taken = true;
    }
}
