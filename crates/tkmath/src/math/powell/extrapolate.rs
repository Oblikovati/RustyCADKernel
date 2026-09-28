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
    pub(super) fn extrapolate(
        &mut self,
        function: &mut impl MultipleVarFunction,
        previous: &mut Vector<'static>,
        direction: &mut Vector<'static>,
        largest: f64,
        column: i32,
    ) -> bool {
        let mut point: Vector<'static> = self.result.location.multiplied(2.0);
        point.subtract(previous).unwrap();
        direction
            .set_difference(&self.result.location, previous)
            .unwrap();
        previous.initialized(&self.result.location).unwrap();
        function.value(&point, &mut self.scratch_value);
        if !self.replace_direction(self.scratch_value, largest) {
            return true;
        }
        if !self.minimize_direction(function, direction) {
            return false;
        }
        for row in 1..=direction.upper() {
            self.directions[(row, column)] = direction[row];
        }
        true
    }
    fn replace_direction(&self, value: f64, largest: f64) -> bool {
        if value.partial_cmp(&self.result.previous) != Some(std::cmp::Ordering::Less) {
            return false;
        }
        let first: f64 = self.result.previous - self.result.minimum - largest;
        let second: f64 = self.result.previous - value;
        let criterion: f64 =
            2.0 * (self.result.previous - 2.0 * self.result.minimum + value) * (first * first)
                - largest * (second * second);
        criterion < 0.0
    }
}
