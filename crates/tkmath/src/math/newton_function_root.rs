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

//! Direct math_NewtonFunctionRoot.cxx at OCCT 3d097a0, retaining the best sampled position.
use super::FunctionWithDerivative;
use rustycad_tkernel::standard::Failure;
mod results;
/// Scalar Newton iteration with independent step and function tolerances; `NewtonFunctionRoot::new(&mut f,guess,1e-8,1e-8)`.
pub struct NewtonFunctionRoot {
    root: f64,
    value: f64,
    derivative: f64,
    iterations: i32,
    limit: i32,
    done: bool,
    lower: f64,
    upper: f64,
    epsilon_x: f64,
    epsilon_f: f64,
}
impl NewtonFunctionRoot {
    /// Solves with the default 100 iterations; `NewtonFunctionRoot::new(&mut f,guess,1e-8,1e-8)`.
    pub fn new(
        function: &mut impl FunctionWithDerivative,
        guess: f64,
        epsilon_x: f64,
        epsilon_f: f64,
    ) -> Self {
        Self::with_iterations(function, guess, epsilon_x, epsilon_f, 100)
    }
    /// Solves without explicit bounds; `NewtonFunctionRoot::with_iterations(&mut f,guess,1e-8,1e-8,100)`.
    pub fn with_iterations(
        function: &mut impl FunctionWithDerivative,
        guess: f64,
        epsilon_x: f64,
        epsilon_f: f64,
        limit: i32,
    ) -> Self {
        Self::bounded(
            function,
            guess,
            epsilon_x,
            epsilon_f,
            -f64::MAX,
            f64::MAX,
            limit,
        )
    }
    /// Solves with native inclusive clipping; `NewtonFunctionRoot::bounded(&mut f,guess,1e-8,1e-8,-2.0,2.0,100)`.
    pub fn bounded(
        function: &mut impl FunctionWithDerivative,
        guess: f64,
        epsilon_x: f64,
        epsilon_f: f64,
        lower: f64,
        upper: f64,
        limit: i32,
    ) -> Self {
        let mut solver: Self = Self::with_bounds(lower, upper, epsilon_x, epsilon_f, limit);
        solver.perform(function, guess);
        solver
    }
    /// Prepares a reusable solver without evaluating; `NewtonFunctionRoot::with_bounds(-2.0,2.0,1e-8,1e-8,100)`.
    pub fn with_bounds(lower: f64, upper: f64, epsilon_x: f64, epsilon_f: f64, limit: i32) -> Self {
        Self {
            root: f64::MAX,
            value: f64::MAX,
            derivative: 0.0,
            iterations: 0,
            limit,
            done: false,
            lower,
            upper,
            epsilon_x,
            epsilon_f,
        }
    }
    /// Repeats native Newton iteration and restores the best sampled root; `solver.perform(&mut f,guess)`.
    pub fn perform(&mut self, function: &mut impl FunctionWithDerivative, guess: f64) {
        let (mut best_x, mut best_value): (f64, f64) = (self.root, f64::MAX);
        let mut step: f64 = f64::MAX;
        self.value = f64::MAX;
        self.root = guess;
        self.iterations = 1;
        while self.iterations <= self.limit
            && (step.abs() > self.epsilon_x || self.value.abs() > self.epsilon_f)
        {
            let good: bool = function.values(self.root, &mut self.value, &mut self.derivative);
            let absolute: f64 = if self.value < 0.0 {
                -self.value
            } else {
                self.value
            };
            if absolute < best_value {
                best_value = absolute;
                best_x = self.root;
            }
            self.advance(good, &mut step);
        }
        self.root = best_x;
        self.done = self.iterations <= self.limit;
    }
    fn advance(&mut self, good: bool, step: &mut f64) {
        if !good || self.derivative == 0.0 {
            self.done = false;
            self.iterations = self.limit + 1;
            return;
        }
        *step = self.value / self.derivative;
        self.root -= *step;
        let (low, high): (f64, f64) = if self.lower < self.upper {
            (self.lower, self.upper)
        } else {
            (self.upper, self.lower)
        };
        if self.root <= low {
            self.root = low;
        }
        if self.root >= high {
            self.root = high;
        }
        self.iterations += 1;
    }
}
impl NewtonFunctionRoot {
    /// Writes the native `Dump` text; `root.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl std::fmt::Write) -> std::fmt::Result {
        use crate::math::stream_real;
        write!(stream, "math_NewtonFunctionRoot ")?;
        if !self.done {
            return writeln!(stream, "Status = not Done ");
        }
        writeln!(stream, " Status = Done ")?;
        writeln!(stream, " Location found = {}", stream_real(self.root))?;
        writeln!(
            stream,
            " function value at this minimum = {}",
            stream_real(self.value)
        )?;
        writeln!(stream, " Number of iterations = {}", self.iterations)
    }
}
display_via_dump!(NewtonFunctionRoot);
