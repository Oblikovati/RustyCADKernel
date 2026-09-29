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

//! General root search with native Newton/gradient selection and boundary handling.
use super::{FunctionSetWithDerivatives, Matrix, Vector};
use rustycad_tkernel::standard::Failure;
use std::fmt;
mod boundary;
mod bounds;
mod descent;
mod direction;
mod engine;
mod evaluate;
mod line;
mod minimize;
mod results;
mod stopping;
/// General equation-system solver; `FunctionSetRoot::new(&function,&tolerances,100)?`.
pub struct FunctionSetRoot {
    sol: Vector<'static>,
    jacobian: Matrix<'static>,
    tol: Vector<'static>,
    delta: Vector<'static>,
    gradient: Vector<'static>,
    direction: Vector<'static>,
    saved_gradient: Vector<'static>,
    saved: Vector<'static>,
    previous: Vector<'static>,
    residual: Vector<'static>,
    lower: Vector<'static>,
    upper: Vector<'static>,
    inverse_lengths: Vector<'static>,
    line_point: Vector<'static>,
    line_residual: Vector<'static>,
    constraints: Vec<bool>,
    history: Vec<f64>,
    done: bool,
    iteration: i32,
    state: i32,
    limit: i32,
    divergent: bool,
    run: Run,
}
#[derive(Default)]
struct Run {
    objective: f64,
    gradient_norm: f64,
    previous_minimum: f64,
    old_gradient: f64,
    old_objective: f64,
    slope: f64,
    lambda: f64,
    lambda2: f64,
    function_tolerance: f64,
    change: bool,
    outside: bool,
    new_solution: bool,
    stop_on_divergent: bool,
    stop: bool,
    good: bool,
    descent: i32,
    boundary: bool,
}
impl FunctionSetRoot {
    /// Configures step tolerances and iteration limit; `FunctionSetRoot::new(&function,&tolerances,100)?`.
    pub fn new(
        function: &impl FunctionSetWithDerivatives,
        tolerance: &Vector<'_>,
        limit: i32,
    ) -> Result<Self, Failure> {
        let mut solver: Self = Self::without_tolerance(function, limit)?;
        solver.set_tolerance(tolerance)?;
        Ok(solver)
    }
    /// Allocates state before tolerances are supplied; `FunctionSetRoot::without_tolerance(&function,100)?`.
    pub fn without_tolerance(
        function: &impl FunctionSetWithDerivatives,
        limit: i32,
    ) -> Result<Self, Failure> {
        let n: i32 = function.variables();
        let m: i32 = function.equations();
        if limit < 0 {
            return Err(Failure::out_of_range(
                "FunctionSetRoot requires a nonnegative iteration limit",
            ));
        }
        Ok(Self {
            sol: vector(n)?,
            // A negative equation count overruns the stack buffer natively (undefined).
            jacobian: Matrix::with_bounds(1, m, 1, n)
                .map_err(|error| error.into_failure("math_FunctionSetRoot"))?,
            tol: vector(n)?,
            delta: vector(n)?,
            gradient: vector(n)?,
            direction: vector(n)?,
            saved_gradient: vector(n)?,
            saved: vector(n)?,
            previous: vector(n)?,
            residual: vector(m)?,
            lower: vector(n)?,
            upper: vector(n)?,
            inverse_lengths: vector(n)?,
            line_point: vector(n)?,
            line_residual: vector(m)?,
            constraints: vec![false; n as usize],
            history: vec![0.0; limit as usize + 1],
            done: false,
            iteration: 0,
            state: 0,
            limit,
            divergent: false,
            run: Run::default(),
        })
    }
    /// Copies declared tolerance indices 1..n; `solver.set_tolerance(&tolerances)?`.
    pub fn set_tolerance(&mut self, tolerance: &Vector<'_>) -> Result<(), Failure> {
        for index in 1..=self.tol.length() {
            self.tol[index] = *tolerance.value(index)?;
        }
        Ok(())
    }
    /// Searches with native finite extreme bounds; `solver.perform(&mut function,&start,false)?`.
    pub fn perform(
        &mut self,
        function: &mut impl FunctionSetWithDerivatives,
        start: &Vector<'_>,
        stop_on_divergent: bool,
    ) -> Result<(), Failure> {
        let lower: Vector<'static> = Vector::filled(1, function.variables(), -f64::MAX)?;
        let upper: Vector<'static> = Vector::filled(1, function.variables(), f64::MAX)?;
        self.perform_bounded(function, start, &lower, &upper, stop_on_divergent)
    }
    /// Searches within supplied bounds and records any attempted divergence; `solver.perform_bounded(&mut function,&start,&lower,&upper,false)?`.
    pub fn perform_bounded<F: FunctionSetWithDerivatives>(
        &mut self,
        function: &mut F,
        start: &Vector<'_>,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
        stop_on_divergent: bool,
    ) -> Result<(), Failure> {
        let mut criterion = |solver: &Self, _: &mut F| solver.is_solution_reached();
        self.perform_with_criterion(
            function,
            start,
            [lower, upper],
            stop_on_divergent,
            &mut criterion,
        )
    }
    /// The bounded search with a replacement for the virtual `IsSolutionReached(F)`; the
    /// criterion reads `current_step`, `current_solution`, `current_jacobian` and
    /// `tolerances`. `solver.perform_with_criterion(&mut f,&start,[&lower,&upper],false,&mut |s, _| s.is_solution_reached())?`.
    pub fn perform_with_criterion<F: FunctionSetWithDerivatives>(
        &mut self,
        function: &mut F,
        start: &Vector<'_>,
        [lower, upper]: [&Vector<'_>; 2],
        stop_on_divergent: bool,
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) -> Result<(), Failure> {
        let n: i32 = function.variables();
        if function.equations() <= 0
            || start.length() != n
            || lower.length() != n
            || upper.length() != n
        {
            return Err(Failure::with_type("Standard_DimensionError", "", ""));
        }
        self.prepare(start, lower, upper, stop_on_divergent)?;
        if self.check_divergence() {
            return Ok(());
        }
        self.clip_initial();
        if !self.evaluate(function) {
            self.finish(function, false, false);
            return Ok(());
        }
        self.run.lambda2 = self.run.gradient_norm;
        self.history[0] = if self.run.objective < 1e-16 {
            1e-16
        } else {
            self.run.objective
        };
        self.run.function_tolerance = if self.run.objective == f64::MAX {
            0.0
        } else {
            self.run.objective.next_up() - self.run.objective
        };
        if self.exact_solution() {
            self.finish(function, true, false);
            return Ok(());
        }
        self.iterate(function, criterion);
        Ok(())
    }
    fn prepare(
        &mut self,
        start: &Vector<'_>,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
        stop: bool,
    ) -> Result<(), Failure> {
        self.done = false;
        self.iteration = 0;
        self.divergent = false;
        self.run = Run {
            stop_on_divergent: stop,
            ..Run::default()
        };
        self.sol.initialized(start)?;
        self.lower.initialized(lower)?;
        self.upper.initialized(upper)?;
        for index in 1..=self.sol.length() {
            let high: f64 = if 2e100 < upper[index] {
                2e100
            } else {
                upper[index]
            };
            let low: f64 = if lower[index] < -2e100 {
                -2e100
            } else {
                lower[index]
            };
            let length: f64 = (high - low) / 4.0;
            self.inverse_lengths[index] = 1.0 / if length < 1e-9 { 1e-9 } else { length };
        }
        Ok(())
    }
    fn clip_initial(&mut self) {
        for index in 1..=self.sol.length() {
            if self.sol[index] <= self.lower[index] {
                self.sol[index] = self.lower[index];
            } else if self.sol[index] > self.upper[index] {
                self.sol[index] = self.upper[index];
            }
        }
    }
    /// Tests the current native step criterion; `solver.is_solution_reached()`.
    pub fn is_solution_reached(&self) -> bool {
        for index in 1..=self.sol.length() {
            if self.delta[index].abs() > self.tol[index] {
                return false;
            }
        }
        true
    }
    /// Protected `Delta`, the last step used by the criterion; `solver.current_step()`.
    pub fn current_step(&self) -> &Vector<'static> {
        &self.delta
    }
    /// Protected `Sol` without the `Done` check; `solver.current_solution()`.
    pub fn current_solution(&self) -> &Vector<'static> {
        &self.sol
    }
    /// Protected `DF`, the current Jacobian; `solver.current_jacobian()`.
    pub fn current_jacobian(&self) -> &Matrix<'static> {
        &self.jacobian
    }
    /// Protected `Tol`, the per-variable step tolerances; `solver.tolerances()`.
    pub fn tolerances(&self) -> &Vector<'static> {
        &self.tol
    }
    /// Writes the native `Dump` text; `solver.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        write!(stream, " math_FunctionSetRoot")?;
        if !self.done {
            return writeln!(stream, "Status = Not Done");
        }
        writeln!(stream, " Status = Done")?;
        write!(stream, " Location value = ")?;
        self.sol.dump(stream)?;
        writeln!(stream)?;
        writeln!(stream, " Number of iterations = {}", self.iteration)
    }
    fn exact_solution(&self) -> bool {
        self.run.objective <= 1e-32 || self.run.gradient_norm <= 1e-64
    }
    fn finish(
        &mut self,
        function: &mut impl FunctionSetWithDerivatives,
        done: bool,
        refresh: bool,
    ) {
        self.done = false;
        if self.run.stop_on_divergent && self.divergent {
            return;
        }
        self.done = done;
        if refresh {
            function.value(&self.sol, &mut self.residual);
        }
        self.state = function.state_number();
    }
}
fn vector(size: i32) -> Result<Vector<'static>, Failure> {
    Vector::with_bounds(1, size)
}
display_via_dump!(FunctionSetRoot);
