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
impl GlobalOptimum<'_> {
    /// Reports the last search status; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Returns the current best value without a Done guard; `solver.minimum()`.
    pub fn minimum(&self) -> f64 {
        self.best
    }
    /// Returns the native candidate count; `solver.number_of_extrema()`.
    pub fn number_of_extrema(&self) -> i32 {
        self.count
    }
    /// Copies a one-based stored solution with native sequence bounds; `solver.points(1,&mut point)?`.
    pub fn points(&self, index: i32, output: &mut Vector<'_>) -> Result<(), Failure> {
        for i in 1..=self.dimension {
            let offset: i64 = (i64::from(index) - 1) * i64::from(self.dimension) + i64::from(i) - 1;
            let value: f64 = *self.solutions.get(offset as usize).ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "solution coordinate {offset}, expected 0..{}",
                    self.solutions.len()
                ))
            })?;
            output[i] = value;
        }
        Ok(())
    }
}
