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

use super::{Failure, MultipleVarFunctionWithHessian, NewtonMinimum, Status, Vector};
impl NewtonMinimum {
    pub(super) fn iterate(
        &mut self,
        function: &mut impl MultipleVarFunctionWithHessian,
        [previous, next]: [&mut Vector<'static>; 2],
        criterion: &mut impl FnMut(&Self) -> bool,
    ) -> Result<(), Failure> {
        let mut consecutive: i32 = 0;
        let mut value: f64 = 0.0;
        loop {
            self.result.iterations += 1;
            if !self.prepare_step(function, previous, &mut value)? {
                return Ok(());
            }
            self.project_step(previous, next);
            self.evaluate_trial(function, previous, next);
            consecutive = if criterion(self) { consecutive + 1 } else { 0 };
            let good: bool = self.correct_trial(function, previous, next, value, consecutive);
            if !good || consecutive >= 2 {
                break;
            }
        }
        self.result.location.initialized(previous)
    }
    fn prepare_step(
        &mut self,
        function: &mut impl MultipleVarFunctionWithHessian,
        point: &Vector<'_>,
        value: &mut f64,
    ) -> Result<bool, Failure> {
        if !function.values_hessian(point, value, &mut self.result.gradient, &mut self.hessian) {
            self.fail(Status::FunctionError);
            return Ok(false);
        }
        if self.result.iterations == 1 {
            self.result.previous = *value;
            self.result.minimum = *value;
        }
        Ok(self.adjust_curvature()? && self.solve_step()?)
    }
}
