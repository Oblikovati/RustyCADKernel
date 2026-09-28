// Copyright (c) 1997-1999 Matra Datavision
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

use super::{FunctionSetRoot, FunctionSetWithDerivatives, direction};
impl FunctionSetRoot {
    pub(super) fn iterate(&mut self, function: &mut impl FunctionSetWithDerivatives) {
        for iteration in 1..=self.limit {
            self.iteration = iteration;
            if !self.initial_step(function) {
                return;
            }
            if (self.run.outside || self.run.objective / self.run.previous_minimum > 0.005)
                && !self.progress(function)
            {
                return;
            }
            if !self.stopping(function) {
                return;
            }
        }
        self.iteration = self.limit.saturating_add(1).max(1);
        self.finish(function, false, false);
    }
    fn initial_step(&mut self, function: &mut impl FunctionSetWithDerivatives) -> bool {
        self.run.previous_minimum = self.run.objective;
        self.run.old_gradient = self.run.gradient_norm;
        self.previous.initialized(&self.sol).unwrap();
        self.saved.initialized(&self.sol).unwrap();
        self.run.slope = direction::search(
            &self.jacobian,
            &self.gradient,
            &self.residual,
            self.run.change,
            &self.inverse_lengths,
            &mut self.direction,
        );
        if self.run.slope.abs() <= 1e-32 {
            self.finish(function, true, true);
            return false;
        }
        self.choose_lambda();
        self.advance();
        if self.check_divergence() {
            return false;
        }
        self.run.outside = self.trim_bounds();
        self.saved_gradient.initialized(&self.gradient).unwrap();
        if !self.evaluate_or_finish(function) {
            return false;
        }
        if self.exact_solution() {
            self.finish(function, true, true);
            return false;
        }
        true
    }
    pub(super) fn choose_lambda(&mut self) {
        if self.run.change {
            self.run.lambda = self.run.lambda2 / self.run.slope.abs().sqrt();
            if self.run.lambda > 1.0 {
                self.run.lambda = 1.0;
            }
        } else {
            self.run.lambda = 1.0;
            self.run.lambda2 = 0.5 * self.run.lambda / self.direction.norm().unwrap();
        }
    }
    pub(super) fn advance(&mut self) {
        for index in 1..=self.sol.length() {
            self.sol[index] += self.run.lambda * self.direction[index];
        }
    }
    fn progress(&mut self, function: &mut impl FunctionSetWithDerivatives) -> bool {
        self.run.slope = self.gradient.dot(&self.direction).unwrap();
        self.run.old_objective = self.run.previous_minimum;
        self.run.stop = false;
        self.run.good = false;
        self.run.descent = 0;
        if !self.run.outside && !self.standard_descent(function) {
            return false;
        }
        if self.run.outside && !self.boundary_descent(function) {
            return false;
        }
        true
    }
}
