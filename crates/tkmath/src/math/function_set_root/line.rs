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

use super::{FunctionSetWithDerivatives, Vector};
use crate::math::Function;
pub(super) struct Line<'a, F> {
    function: &'a mut F,
    origin: Vector<'static>,
    direction: Vector<'static>,
    point: &'a mut Vector<'static>,
    residual: &'a mut Vector<'static>,
}
impl<'a, F: FunctionSetWithDerivatives> Line<'a, F> {
    pub(super) fn new(
        function: &'a mut F,
        origin: &Vector<'static>,
        direction: &Vector<'static>,
        point: &'a mut Vector<'static>,
        residual: &'a mut Vector<'static>,
    ) -> Self {
        Self {
            function,
            origin: origin.clone(),
            direction: direction.clone(),
            point,
            residual,
        }
    }
}
impl<F: FunctionSetWithDerivatives> Function for Line<'_, F> {
    fn value(&mut self, parameter: f64, value: &mut f64) -> bool {
        for index in 1..=self.point.length() {
            self.point[index] = self.direction[index] * parameter + self.origin[index];
        }
        if !self.function.value(self.point, self.residual) {
            return false;
        }
        for index in 1..=self.residual.length() {
            if self.residual[index] <= -1e100 || self.residual[index] >= 1e100 {
                return false;
            }
        }
        *value = 0.5 * self.residual.norm2_unchecked();
        true
    }
}
