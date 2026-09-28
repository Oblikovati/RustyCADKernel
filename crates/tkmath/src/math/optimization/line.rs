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

//! Shared OCCT line callback: multiply before adding and reset the objective output.
use super::super::{Function, MultipleVarFunction, Vector};
pub(crate) struct ObjectiveLine<'a, F> {
    pub function: &'a mut F,
    pub origin: &'a Vector<'static>,
    pub direction: &'a Vector<'static>,
    pub point: Vector<'static>,
}
impl<F: MultipleVarFunction> Function for ObjectiveLine<'_, F> {
    fn value(&mut self, scale: f64, result: &mut f64) -> bool {
        self.point.initialized(self.direction).unwrap();
        self.point.multiply(scale);
        self.point.add(self.origin).unwrap();
        *result = 0.0;
        self.function.value(&self.point, result)
    }
}

pub(in crate::math) fn minimize(
    function: &mut impl MultipleVarFunction,
    point: &mut Vector<'static>,
    direction: &mut Vector<'static>,
    value: &mut f64,
) -> bool {
    let mut line: ObjectiveLine<'_, _> = ObjectiveLine {
        function,
        origin: point,
        direction,
        point: Vector::with_bounds(1, direction.upper()).unwrap(),
    };
    let Some((scale, minimum)) = search(&mut line) else {
        return false;
    };
    *value = minimum;
    direction.multiply(scale);
    point.add(direction).unwrap();
    true
}
fn search(function: &mut impl Function) -> Option<(f64, f64)> {
    let bracket: super::super::BracketMinimum =
        super::super::BracketMinimum::compute(function, 0.0, 1.0);
    if !bracket.is_done() {
        return None;
    }
    let [first, middle, last]: [f64; 3] = bracket.values().unwrap();
    let mut solver: super::super::BrentMinimum = super::super::BrentMinimum::new(1e-10);
    solver.perform(function, first, middle, last);
    if !solver.is_done() {
        return None;
    }
    Some((solver.location().unwrap(), solver.minimum().unwrap()))
}
