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

use super::{MultipleVarFunction, Powell, Vector};
impl Powell {
    pub(super) fn iteration(
        &mut self,
        function: &mut impl MultipleVarFunction,
        previous: &mut Vector<'static>,
        direction: &mut Vector<'static>,
    ) -> bool {
        function.value(&self.result.location, &mut self.result.previous);
        let Some((largest, column)) = self.search_columns(function, direction) else {
            return true;
        };
        if self.is_solution_reached() {
            self.state = function.state_number();
            self.result.done = true;
            return true;
        }
        if self.result.iterations == self.result.limit {
            return true;
        }
        !self.extrapolate(function, previous, direction, largest, column)
    }
    fn search_columns(
        &mut self,
        function: &mut impl MultipleVarFunction,
        direction: &mut Vector<'static>,
    ) -> Option<(f64, i32)> {
        let (mut largest, mut column): (f64, i32) = (0.0, 0);
        for index in 1..=direction.upper() {
            for row in 1..=direction.upper() {
                direction[row] = self.directions[(row, index)];
            }
            function.value(&self.result.location, &mut self.scratch_value);
            if !self.minimize_direction(function, direction) {
                return None;
            }
            if (self.scratch_value - self.result.minimum).abs() > largest {
                largest = (self.scratch_value - self.result.minimum).abs();
                column = index;
            }
        }
        Some((largest, column))
    }
}
