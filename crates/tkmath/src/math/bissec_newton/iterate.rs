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

use super::{BissecNewton, FunctionWithDerivative};
impl BissecNewton {
    pub(super) fn iterate(
        &mut self,
        function: &mut impl FunctionWithDerivative,
        mut low: f64,
        mut high: f64,
        limit: i32,
    ) {
        let mut old_step: f64 = self.dx;
        for _ in 1..=limit {
            if self.step(low, high, &mut old_step) || self.is_solution_reached() {
                self.done = true;
                return;
            }
            if !function.values(self.x, &mut self.value, &mut self.derivative) {
                return;
            }
            if self.update_bracket(&mut low, &mut high) {
                self.done = true;
                return;
            }
        }
    }
    fn step(&mut self, low: f64, high: f64, old_step: &mut f64) -> bool {
        let outside: bool = ((self.x - high) * self.derivative - self.value)
            * ((self.x - low) * self.derivative - self.value)
            >= 0.0;
        let slow: bool = (2.0 * self.value).abs() > (*old_step * self.derivative).abs();
        *old_step = self.dx;
        if outside || slow {
            self.dx = 0.5 * (high - low);
            self.x = low + self.dx;
            return self.dx.abs() < self.tolerance;
        }
        self.dx = self.value / self.derivative;
        let previous: f64 = self.x;
        self.x -= self.dx;
        previous == self.x
    }
    fn update_bracket(&self, low: &mut f64, high: &mut f64) -> bool {
        if self.value < 0.0 {
            *low = self.x;
            return false;
        }
        if self.value > 0.0 {
            *high = self.x;
            return false;
        }
        true
    }
}
