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

use super::{Bfgs, Matrix, MultipleVarFunctionWithGradient, Vector};
impl Bfgs {
    pub(super) fn iterate(
        &mut self,
        function: &mut impl MultipleVarFunctionWithGradient,
        direction: &mut Vector<'static>,
        hessian: &mut Matrix<'static>,
    ) {
        self.result.iterations = 1;
        while self.result.iterations <= self.result.limit {
            if self.iteration(function, direction, hessian) {
                return;
            }
            self.result.iterations += 1;
        }
        self.result.done = false;
    }
    fn iteration(
        &mut self,
        function: &mut impl MultipleVarFunctionWithGradient,
        direction: &mut Vector<'static>,
        hessian: &mut Matrix<'static>,
    ) -> bool {
        self.result.minimum = self.result.previous;
        let good: bool = self.minimize_direction(function, direction);
        // OCC30492: even a failed direction can satisfy the objective stopping test.
        if self.result.reached() {
            self.result.done = true;
            return true;
        }
        if !good {
            self.result.done = false;
            return true;
        }
        self.result.previous = self.result.minimum;
        let mut difference: Vector<'static> = self.result.gradient.clone();
        if !function.values(
            &self.result.location,
            &mut self.result.minimum,
            &mut self.result.gradient,
        ) {
            self.result.done = false;
            return true;
        }
        for index in 1..=difference.upper() {
            difference[index] = self.result.gradient[index] - difference[index];
        }
        super::update::update_hessian(hessian, direction, &mut difference);
        super::update::next_direction(hessian, &self.result.gradient, direction);
        false
    }
}
