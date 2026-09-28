// Created on: 2014-01-20
// Created by: Alexaner Malyshev
// Copyright (c) 2014-2015 OPEN CASCADE SAS
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
// commercial license or contractual agreement

use super::{Failure, GlobalOptimum, Vector};
use crate::math::{Bfgs, Matrix, NewtonMinimum, Powell};
impl GlobalOptimum<'_> {
    pub(super) fn local(
        &mut self,
        start: &Vector<'_>,
        value: &mut f64,
        output: &mut Vector<'_>,
    ) -> Result<bool, Failure> {
        if self.continuity >= 2
            && self.function.has_hessian()
            && self.newton(start, value, output)?
        {
            return Ok(true);
        }
        if self.continuity >= 1
            && self.function.has_gradient()
            && self.bfgs(start, value, output)?
        {
            return Ok(true);
        }
        let mut directions: Matrix<'static> =
            Matrix::filled(1, self.dimension, 1, self.dimension, 0.0).unwrap();
        for i in 1..=self.dimension {
            directions[(i, i)] = 1.0;
        }
        let mut solver: Powell = Powell::new(&self.function, 1e-10)?;
        solver.perform(&mut self.function, start, &directions)?;
        if !solver.is_done() {
            return Ok(false);
        }
        solver.copy_location(output)?;
        *value = solver.minimum()?;
        Ok(self.inside(output))
    }
    fn newton(
        &mut self,
        start: &Vector<'_>,
        value: &mut f64,
        output: &mut Vector<'_>,
    ) -> Result<bool, Failure> {
        let mut solver: NewtonMinimum = NewtonMinimum::new(&self.function)?;
        solver.set_boundary(&self.global_lower, &self.global_upper)?;
        solver.perform(&mut self.function, start)?;
        if !solver.is_done() {
            return Ok(false);
        }
        solver.copy_location(output)?;
        *value = solver.minimum()?;
        Ok(self.inside(output))
    }
    fn bfgs(
        &mut self,
        start: &Vector<'_>,
        value: &mut f64,
        output: &mut Vector<'_>,
    ) -> Result<bool, Failure> {
        let mut solver: Bfgs = Bfgs::new(self.dimension)?;
        solver.set_boundary(&self.global_lower, &self.global_upper)?;
        solver.perform(&mut self.function, start)?;
        if !solver.is_done() {
            return Ok(false);
        }
        solver.copy_location(output)?;
        *value = solver.minimum()?;
        Ok(self.inside(output))
    }
    fn inside(&self, point: &Vector<'_>) -> bool {
        !(1..=self.dimension)
            .any(|i| point[i] < self.global_lower[i] || point[i] > self.global_upper[i])
    }
}
