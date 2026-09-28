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

use super::{Failure, GlobalObjective, GlobalOptimum, Vector};
impl<'a> GlobalOptimum<'a> {
    /// Replaces parameters without clearing the native stored solution buffer; `solver.set_global_params(objective,&low,&high,9.0,1e-2,1e-7)?`.
    pub fn set_global_params(
        &mut self,
        function: GlobalObjective<'a>,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
        constant: f64,
        tolerance: f64,
        same: f64,
    ) -> Result<(), Failure> {
        self.function = function;
        self.constant = constant;
        self.initial_constant = constant;
        self.count = 0;
        self.copy_global_bounds(lower, upper)?;
        self.tolerance = tolerance;
        self.same = same;
        self.initial_solutions()?;
        self.done = false;
        Ok(())
    }
    pub(super) fn copy_global_bounds(
        &mut self,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
    ) -> Result<(), Failure> {
        for i in 1..=self.dimension {
            self.global_lower[i] = lower[i];
            self.global_upper[i] = upper[i];
        }
        self.set_local_params(lower, upper)
    }
    /// Sets the local search box while retaining global bounds and candidates; `solver.set_local_params(&low,&high)?`.
    pub fn set_local_params(
        &mut self,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
    ) -> Result<(), Failure> {
        for i in 1..=self.dimension {
            self.lower[i] = lower[i];
            self.upper[i] = upper[i];
            self.max_steps[i] = (upper[i] - lower[i]) / 3.0;
        }
        self.done = false;
        Ok(())
    }
    /// Changes both native tolerances; `solver.set_tolerances(1e-2,1e-7)`.
    pub fn set_tolerances(&mut self, discretization: f64, same: f64) {
        self.tolerance = discretization;
        self.same = same;
    }
    /// Reads `[discretization,same]` tolerances; `solver.tolerances()`.
    pub fn tolerances(&self) -> [f64; 2] {
        [self.tolerance, self.same]
    }
    /// Limits local optimization's derivative order; `solver.set_continuity(1)`.
    pub fn set_continuity(&mut self, continuity: i32) {
        self.continuity = continuity;
    }
    /// Reads the derivative continuity setting; `solver.continuity()`.
    pub fn continuity(&self) -> i32 {
        self.continuity
    }
    /// Supplies the desired functional minimum; `solver.set_functional_minimal_value(0.0)`.
    pub fn set_functional_minimal_value(&mut self, value: f64) {
        self.minimal = value;
    }
    /// Reads the desired functional minimum; `solver.functional_minimal_value()`.
    pub fn functional_minimal_value(&self) -> f64 {
        self.minimal
    }
    /// Locks or unlocks Lipschitz estimation; `solver.set_lipschitz_locked(true)`.
    pub fn set_lipschitz_locked(&mut self, locked: bool) {
        self.locked = locked;
    }
    /// Reads the Lipschitz lock state; `solver.lipschitz_locked()`.
    pub fn lipschitz_locked(&self) -> bool {
        self.locked
    }
}
