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

//! Shared storage and guarded result access for the native gradient optimizers.
use super::{MultipleVarFunctionWithGradient, Vector};
use rustycad_tkernel::standard::Failure;
pub(super) struct Optimization {
    pub location: Vector<'static>,
    pub gradient: Vector<'static>,
    pub minimum: f64,
    pub previous: f64,
    pub tolerance: f64,
    pub epsilon: f64,
    pub iterations: i32,
    pub limit: i32,
    pub done: bool,
    pub failure_message: &'static str,
}
impl Optimization {
    pub fn new(size: i32, tolerance: f64, limit: i32, epsilon: f64) -> Result<Self, Failure> {
        Ok(Self {
            location: Vector::with_bounds(1, size)?,
            gradient: Vector::with_bounds(1, size)?,
            minimum: 0.0,
            previous: 0.0,
            tolerance,
            epsilon,
            iterations: 0,
            limit,
            done: false,
            failure_message: " ",
        })
    }
    pub fn initialize(
        &mut self,
        function: &mut impl MultipleVarFunctionWithGradient,
        start: &Vector<'_>,
    ) -> Result<bool, Failure> {
        self.location.initialized(start)?;
        if function.values(&self.location, &mut self.previous, &mut self.gradient) {
            return Ok(true);
        }
        self.done = false;
        Ok(false)
    }
    pub fn reached(&self) -> bool {
        2.0 * (self.minimum - self.previous).abs()
            <= self.tolerance * (self.minimum.abs() + self.previous.abs() + self.epsilon)
    }
    pub fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                self.failure_message,
                "",
            ));
        }
        Ok(())
    }
}
macro_rules! minimum_results {
    ($solver:ty) => {
        impl $solver {
            /// Reports native convergence; `solver.is_done()`.
            pub fn is_done(&self) -> bool {
                self.result.done
            }
            /// Borrows the minimum location after convergence; `solver.location()?`.
            pub fn location(&self) -> Result<&Vector<'static>, Failure> {
                self.result.check_done()?;
                Ok(&self.result.location)
            }
            /// Copies the minimum location, preserving destination bounds; `solver.copy_location(&mut output)?`.
            pub fn copy_location(&self, output: &mut Vector<'_>) -> Result<(), Failure> {
                self.result.check_done()?;
                output.initialized(&self.result.location)
            }
            /// Reads the cached objective; `solver.minimum()?`.
            pub fn minimum(&self) -> Result<f64, Failure> {
                self.result.check_done()?;
                Ok(self.result.minimum)
            }
            /// Reads the native iteration counter; `solver.iterations()?`.
            pub fn iterations(&self) -> Result<i32, Failure> {
                self.result.check_done()?;
                Ok(self.result.iterations)
            }
        }
    };
}
pub(super) use minimum_results;
macro_rules! optimization_results {
    ($solver:ty) => {
        super::optimization::gradient_results!($solver);
        impl $solver {
            /// Applies the default native convergence criterion; `solver.is_solution_reached()`.
            pub fn is_solution_reached(&self) -> bool {
                self.result.reached()
            }
        }
    };
}
pub(super) use optimization_results;
pub(super) mod line;

macro_rules! gradient_results {
    ($solver:ty) => {
        super::optimization::minimum_results!($solver);
        impl $solver {
            /// Borrows the native cached gradient or search direction; `solver.gradient()?`.
            pub fn gradient(&self) -> Result<&Vector<'static>, Failure> {
                self.result.check_done()?;
                Ok(&self.result.gradient)
            }
            /// Copies the cached gradient, preserving destination bounds; `solver.copy_gradient(&mut output)?`.
            pub fn copy_gradient(&self, output: &mut Vector<'_>) -> Result<(), Failure> {
                self.result.check_done()?;
                output.initialized(&self.result.gradient)
            }
        }
    };
}
pub(super) use gradient_results;
