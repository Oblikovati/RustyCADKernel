//! Brent minimization preserving OCCT's step order and persistent done state.
use super::{Function, stream_real};
use rustycad_tkernel::standard::Failure;
use std::fmt;
mod search;
mod step;
#[derive(Clone, Copy)]
struct Sample {
    x: f64,
    value: f64,
}
struct History {
    v: Sample,
    w: Sample,
    d: f64,
    e: f64,
}
/// Stateful Brent minimizer; `BrentMinimum::new(1e-10)`.
pub struct BrentMinimum {
    left: f64,
    right: f64,
    current: Sample,
    tolerance: f64,
    epsilon: f64,
    done: bool,
    iterations: i32,
    limit: i32,
    known: bool,
}
impl BrentMinimum {
    /// Uses native defaults of 100 iterations and ZEPS=1e-12; `BrentMinimum::new(1e-10)`.
    pub fn new(tolerance: f64) -> Self {
        Self::with_options(tolerance, 100, 1e-12)
    }
    /// Configures iteration and absolute tolerance; `BrentMinimum::with_options(1e-10,200,1e-15)`.
    pub fn with_options(tolerance: f64, limit: i32, epsilon: f64) -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            current: Sample { x: 0.0, value: 0.0 },
            tolerance,
            epsilon,
            done: false,
            iterations: 0,
            limit,
            known: false,
        }
    }
    /// Supplies the middle function value for subsequent Perform calls; `BrentMinimum::with_value(1e-10,1.25,100,1e-12)`.
    pub fn with_value(tolerance: f64, value: f64, limit: i32, epsilon: f64) -> Self {
        let mut solver: Self = Self::with_options(tolerance, limit, epsilon);
        solver.current.value = value;
        solver.known = true;
        solver
    }
    /// Runs the native minimization on a supplied triplet; `solver.perform(&mut function,0.0,1.5,4.0)`.
    pub fn perform<F: Function>(&mut self, function: &mut F, first: f64, middle: f64, last: f64) {
        let mut criterion = |solver: &Self, _: &mut F| solver.is_solution_reached();
        self.perform_with_criterion(function, [first, middle, last], &mut criterion);
    }
    /// Runs with a replacement for the virtual `IsSolutionReached(F)`; the criterion
    /// reads `bracket`, `current_location`, `current_value`, `tolerance` and
    /// `zero_epsilon`. `solver.perform_with_criterion(&mut f,[0.0,1.5,4.0],&mut |s, _| s.is_solution_reached())`.
    pub fn perform_with_criterion<F: Function>(
        &mut self,
        function: &mut F,
        [first, middle, last]: [f64; 3],
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) {
        self.left = if first < last { first } else { last };
        self.right = if first > last { first } else { last };
        self.current.x = middle;
        // Native does not clear Done before callbacks; an early failure can retain an earlier success.
        if !self.known && !function.value(middle, &mut self.current.value) {
            return;
        }
        let mut history: History = History {
            v: self.current,
            w: self.current,
            d: f64::MAX,
            e: 0.0,
        };
        self.iterate(function, &mut history, criterion);
    }
    /// Applies the default native convergence test; `solver.is_solution_reached()`.
    pub fn is_solution_reached(&self) -> bool {
        let twice: f64 = 2.0 * (self.tolerance * self.current.x.abs() + self.epsilon);
        self.current.x <= twice + self.left && self.current.x >= self.right - twice
    }
    /// Reports the native completion flag; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reads the minimizing position after convergence; `solver.location()?`.
    pub fn location(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.current.x)
    }
    /// Reads the cached minimum value; `solver.minimum()?`.
    pub fn minimum(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.current.value)
    }
    /// Reads the current native iteration count; `solver.iterations()?`.
    pub fn iterations(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.iterations)
    }
    /// Protected bracket `[a, b]` of the running search; `solver.bracket()`.
    pub fn bracket(&self) -> [f64; 2] {
        [self.left, self.right]
    }
    /// Protected `x`, the best abscissa so far; `solver.current_location()`.
    pub fn current_location(&self) -> f64 {
        self.current.x
    }
    /// Protected `fx`, the value at `x`; `solver.current_value()`.
    pub fn current_value(&self) -> f64 {
        self.current.value
    }
    /// Protected `XTol`; `solver.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
    /// Protected `EPSZ`; `solver.zero_epsilon()`.
    pub fn zero_epsilon(&self) -> f64 {
        self.epsilon
    }
    /// Writes the native `Dump` text; `solver.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        write!(stream, "math_BrentMinimum ")?;
        if !self.done {
            return writeln!(stream, " Status = not Done ");
        }
        writeln!(stream, " Status = Done ")?;
        writeln!(stream, " Location value = {}", stream_real(self.current.x))?;
        writeln!(
            stream,
            " Minimum value = {}",
            stream_real(self.current.value)
        )?;
        writeln!(stream, " Number of iterations = {}", self.iterations)
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
display_via_dump!(BrentMinimum);
