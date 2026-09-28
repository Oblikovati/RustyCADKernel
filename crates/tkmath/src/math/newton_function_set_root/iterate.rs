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

use super::{Failure, FunctionSetWithDerivatives, Gauss, NewtonFunctionSetRoot, Vector};
impl NewtonFunctionSetRoot {
    pub(super) fn iterate(
        &mut self,
        function: &mut impl FunctionSetWithDerivatives,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
    ) -> Result<(), Failure> {
        for iteration in 1..=self.limit {
            self.iteration = iteration;
            if !self.newton_step(lower, upper)? {
                return Ok(());
            }
            if !function.values(&self.solution, &mut self.residual, &mut self.jacobian) {
                return Ok(());
            }
            if self.is_solution_reached() {
                self._state = function.state_number();
                self.done = true;
                return Ok(());
            }
        }
        self.iteration = self.limit.saturating_add(1).max(1);
        Ok(())
    }
    fn newton_step(&mut self, lower: &Vector<'_>, upper: &Vector<'_>) -> Result<bool, Failure> {
        for index in 1..=self.delta.length() {
            self.delta[index] = -self.residual[index];
        }
        let factor: Gauss = Gauss::with_min_pivot(&self.jacobian, 1e-30)?;
        self.jacobian.initialized(factor.factor_matrix())?;
        if !factor.is_done() {
            return Ok(false);
        }
        factor.solve(&mut self.delta)?;
        self.clip_trial(lower, upper)?;
        Ok(true)
    }
    fn clip_trial(&mut self, lower: &Vector<'_>, upper: &Vector<'_>) -> Result<(), Failure> {
        for index in 1..=self.solution.length() {
            self.solution[index] += self.delta[index];
            let low: f64 = *lower.value(index)?;
            let high: f64 = *upper.value(index)?;
            if self.solution[index] <= low {
                self.solution[index] = low;
            }
            if self.solution[index] >= high {
                self.solution[index] = high;
            }
        }
        Ok(())
    }
}
