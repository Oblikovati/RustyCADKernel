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
use super::{NewtonState, owned};
use crate::{
    math::{Matrix, Vector},
    math_lin::utils::is_finite_vector,
    math_sys::linear::{LinearStep, RANK_TOLERANCE, safe_norm, solve_linearized, zero_matrix},
    math_utils::Status,
};
impl NewtonState {
    pub(super) fn direction(
        &mut self,
    ) -> Option<(Vector<'static>, Vector<'static>, Vector<'static>)> {
        let (rhs, gradient): (Vector<'static>, Vector<'static>) = self.gradient_rhs();
        if !is_finite_vector(&gradient) {
            self.result.status = Status::NumericalError;
            return None;
        }
        let step: LinearStep = solve_linearized(&self.jacobian, &rhs, RANK_TOLERANCE);
        self.result.rank = step.rank;
        let Some(mut direction) = step.solution else {
            self.result.status = step.status;
            return None;
        };
        let mut search_gradient: Vector<'static> = owned(&gradient);
        if !self.project_direction(&rhs, &mut direction, &mut search_gradient) {
            return None;
        }
        Some((direction, gradient, search_gradient))
    }
    fn gradient_rhs(&self) -> (Vector<'static>, Vector<'static>) {
        let mut rhs: Vector<'static> = Vector::with_size(self.residual.length() as usize);
        let mut gradient: Vector<'static> = Vector::with_size(self.solution.length() as usize);
        for row in 0..self.residual.length() as usize {
            *rhs.change_at(row).unwrap() = -self.residual.at(row).unwrap();
            for col in 0..self.solution.length() as usize {
                *gradient.change_at(col).unwrap() +=
                    self.jacobian.at(row, col).unwrap() * self.residual.at(row).unwrap();
            }
        }
        (rhs, gradient)
    }
    fn free_variables(&self, gradient: &mut Vector<'_>) -> Vec<usize> {
        let (lower, upper) = self.bounds.as_ref().unwrap();
        let mut free: Vec<usize> = Vec::new();
        for i in 0..self.solution.length() as usize {
            let x: f64 = *self.solution.at(i).unwrap();
            let g: f64 = *gradient.at(i).unwrap();
            let blocked: bool = (x <= *lower.at(i).unwrap() && g >= 0.0)
                || (x >= *upper.at(i).unwrap() && g <= 0.0)
                || lower.at(i).unwrap() == upper.at(i).unwrap();
            if blocked {
                *gradient.change_at(i).unwrap() = 0.0;
            } else {
                free.push(i);
            }
        }
        free
    }
    fn project_direction(
        &mut self,
        rhs: &Vector<'_>,
        direction: &mut Vector<'_>,
        gradient: &mut Vector<'_>,
    ) -> bool {
        if self.bounds.is_none() {
            return true;
        }
        let free: Vec<usize> = self.free_variables(gradient);
        if free.len() == self.solution.length() as usize {
            return true;
        }
        if free.is_empty() {
            self.result.status = Status::NotConverged;
            return false;
        }
        let reduced: Matrix<'static> = self.reduced_jacobian(&free);
        let step: LinearStep = solve_linearized(&reduced, rhs, RANK_TOLERANCE);
        direction.init(0.0);
        if let Some(solution) = step.solution {
            for (i, &col) in free.iter().enumerate() {
                *direction.change_at(col).unwrap() = *solution.at(i).unwrap();
            }
        } else {
            for col in 0..self.solution.length() as usize {
                *direction.change_at(col).unwrap() = -gradient.at(col).unwrap();
            }
        }
        true
    }
    fn reduced_jacobian(&self, free: &[usize]) -> Matrix<'static> {
        let mut reduced: Matrix<'static> = zero_matrix(self.residual.length() as usize, free.len());
        for row in 0..self.residual.length() as usize {
            for (i, &col) in free.iter().enumerate() {
                *reduced.change_at(row, i).unwrap() = *self.jacobian.at(row, col).unwrap();
            }
        }
        reduced
    }
    pub(super) fn ensure_descent(
        &mut self,
        direction: &mut Vector<'_>,
        gradient: &Vector<'_>,
        norm: f64,
    ) -> bool {
        let dot: f64 = gradient
            .values()
            .iter()
            .zip(direction.values())
            .fold(0.0, |sum, (g, d)| sum + g * d);
        if dot < 0.0 {
            return true;
        }
        let gradient_norm: f64 = safe_norm(gradient);
        if gradient_norm.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
            self.stationary_step(norm);
            return false;
        }
        for col in 0..direction.length() as usize {
            *direction.change_at(col).unwrap() = -gradient.at(col).unwrap() / gradient_norm;
        }
        true
    }
    fn stationary_step(&mut self, norm: f64) {
        if norm <= self.f_tolerance {
            self.result.iterations += 1;
            self.result.step_norm = 0.0;
            self.taken = true;
            self.within_x = true;
            self.result.status = Status::Ok;
            return;
        }
        self.result.status =
            if self.result.rank < self.residual.length().min(self.solution.length()) as usize {
                Status::Singular
            } else {
                Status::NotConverged
            };
    }
}
