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

use super::{FunctionSetRoot, FunctionSetWithDerivatives};
impl FunctionSetRoot {
    pub(super) fn stopping(&mut self, function: &mut impl FunctionSetWithDerivatives) -> bool {
        self.history[self.iteration as usize] = self.run.objective;
        let verify: bool = self.should_verify();
        if verify && self.step_is_small() {
            if self.run.previous_minimum < self.run.objective {
                self.sol.initialized(&self.saved).unwrap();
            }
            self.finish(function, true, true);
            return false;
        }
        if self.run.objective - self.run.previous_minimum <= self.run.function_tolerance {
            return self.analyze_progress(function, verify);
        }
        if self.run.change {
            self.finish(function, false, false);
            return false;
        }
        self.run.change = true;
        self.sol.initialized(&self.previous).unwrap();
        if !self.evaluate(function) {
            self.finish(function, false, false);
            return false;
        }
        true
    }
    fn should_verify(&self) -> bool {
        if self.run.change {
            return true;
        }
        if self.iteration > 1 {
            return self.history[self.iteration as usize - 1]
                < 1e-4 * self.history[self.iteration as usize - 2];
        }
        self.run.objective < 1e-6 * self.history[0]
    }
    fn step_is_small(&mut self) -> bool {
        for index in 1..=self.delta.length() {
            self.delta[index] = self.previous[index] - self.sol[index];
        }
        self.is_solution_reached()
    }
    fn analyze_progress(
        &mut self,
        function: &mut impl FunctionSetWithDerivatives,
        verified: bool,
    ) -> bool {
        if self.iteration > 5
            && self.run.objective >= 0.95 * self.history[self.iteration as usize - 5]
        {
            if self.run.change {
                self.finish(function, true, false);
                return false;
            }
            self.run.change = true;
        } else {
            self.run.change = false;
        }
        if self.run.gradient_norm > 0.9 * self.run.old_gradient
            && self.run.objective > 0.5 * self.run.previous_minimum
        {
            self.run.change = true;
        }
        if !self.run.change && !verified && self.step_is_small() {
            self.finish(function, true, true);
            return false;
        }
        true
    }
}
