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

use super::{Frpr, MultipleVarFunctionWithGradient, Vector};
impl Frpr {
    pub(super) fn iterate(
        &mut self,
        function: &mut impl MultipleVarFunctionWithGradient,
        negative: &mut Vector<'static>,
        previous_direction: &mut Vector<'static>,
    ) {
        for iteration in 1..=self.result.limit {
            self.result.iterations = iteration;
            if self.iteration(function, negative, previous_direction) {
                return;
            }
        }
        self.result.done = false;
    }
    fn iteration(
        &mut self,
        function: &mut impl MultipleVarFunctionWithGradient,
        negative: &mut Vector<'static>,
        previous_direction: &mut Vector<'static>,
    ) -> bool {
        let good: bool = self.minimize_direction(function);
        if self.result.reached() {
            self.result.done = true;
            self.state = function.state_number();
            return true;
        }
        if !good {
            self.result.done = false;
            return true;
        }
        if !function.values(
            &self.result.location,
            &mut self.result.previous,
            &mut self.result.gradient,
        ) {
            self.result.done = false;
            return true;
        }
        if !next_direction(&mut self.result.gradient, negative, previous_direction) {
            self.result.done = false;
            return true;
        }
        false
    }
}
fn next_direction(
    gradient: &mut Vector<'static>,
    negative: &mut Vector<'static>,
    previous: &mut Vector<'static>,
) -> bool {
    let (mut gg, mut dgg): (f64, f64) = (0.0, 0.0);
    for index in 1..=negative.upper() {
        gg += negative[index] * negative[index];
        dgg += (gradient[index] + negative[index]) * gradient[index];
    }
    if gg == 0.0 {
        return false;
    }
    let gamma: f64 = dgg / gg;
    for index in 1..=negative.upper() {
        negative[index] = -gradient[index];
        gradient[index] = negative[index] + gamma * previous[index];
    }
    previous.initialized(gradient).unwrap();
    true
}
