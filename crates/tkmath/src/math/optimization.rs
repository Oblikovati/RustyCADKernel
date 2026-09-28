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
use super::{MultipleVarFunctionWithGradient, Status, Vector, stream_real};
use rustycad_tkernel::standard::Failure;
use std::fmt;
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
    pub status: Status,
    pub failure_message: &'static str,
}
impl Optimization {
    /// `status` is the constructor's initial `TheStatus` (math_OK for BFGS,
    /// math_NotBracketed for FRPR and Powell).
    pub fn new(
        size: i32,
        [tolerance, epsilon]: [f64; 2],
        limit: i32,
        status: Status,
    ) -> Result<Self, Failure> {
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
            status,
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
        self.fail(Status::FunctionError);
        Ok(false)
    }
    /// Records a native `Done = true; TheStatus = math_OK` exit.
    pub fn succeed(&mut self) {
        self.done = true;
        self.status = Status::Ok;
    }
    /// Records a native `Done = false; TheStatus = status` exit.
    pub fn fail(&mut self, status: Status) {
        self.done = false;
        self.status = status;
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
    /// The shared native Dump body after the class-specific `prefix`.
    pub fn dump(&self, stream: &mut impl fmt::Write, prefix: &str) -> fmt::Result {
        write!(stream, "{prefix}")?;
        if !self.done {
            return writeln!(stream, " Status = not Done because {}", self.status as i32);
        }
        writeln!(stream, " Status = Done ")?;
        write!(stream, " Location Vector = ")?;
        self.location.dump(stream)?;
        writeln!(stream)?;
        writeln!(stream, " Minimum value = {}", stream_real(self.minimum))?;
        writeln!(stream, " Number of iterations = {}", self.iterations)
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
            /// Protected `TheMinimum`, readable by convergence criteria before `Done`;
            /// `solver.current_minimum()`.
            pub fn current_minimum(&self) -> f64 {
                self.result.minimum
            }
            /// Protected `PreviousMinimum`; `solver.previous_minimum()`.
            pub fn previous_minimum(&self) -> f64 {
                self.result.previous
            }
            /// Protected `TheLocation` without the `Done` check; `solver.current_location()`.
            pub fn current_location(&self) -> &Vector<'static> {
                &self.result.location
            }
            /// Protected `XTol`, the relative objective tolerance; `solver.tolerance()`.
            pub fn tolerance(&self) -> f64 {
                self.result.tolerance
            }
            /// Protected `EPSZ`, the absolute zero guard; `solver.zero_epsilon()`.
            pub fn zero_epsilon(&self) -> f64 {
                self.result.epsilon
            }
            /// `TheStatus` of the last `perform`, readable even when not done;
            /// `solver.status()`.
            pub fn status(&self) -> Status {
                self.result.status
            }
        }
    };
}
pub(super) use minimum_results;
macro_rules! standard_dump {
    ($solver:ty, $prefix:literal) => {
        impl $solver {
            /// Writes the native `Dump` text; `solver.dump(&mut text)?`.
            pub fn dump(&self, stream: &mut impl std::fmt::Write) -> std::fmt::Result {
                self.result.dump(stream, $prefix)
            }
        }
        display_via_dump!($solver);
    };
}
pub(super) use standard_dump;
macro_rules! optimization_results {
    ($solver:ty) => {
        super::optimization::gradient_results!($solver);
        impl $solver {
            /// Applies the default native convergence criterion (the base-class
            /// `IsSolutionReached`); `solver.is_solution_reached()`.
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
            /// Protected `TheGradient` without the `Done` check; `solver.current_gradient()`.
            pub fn current_gradient(&self) -> &Vector<'static> {
                &self.result.gradient
            }
        }
    };
}
pub(super) use gradient_results;
