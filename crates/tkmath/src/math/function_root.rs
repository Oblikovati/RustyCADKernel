//! One-variable adapter around the native general function-set root algorithm.
use super::{
    FunctionSet, FunctionSetRoot, FunctionSetWithDerivatives, FunctionWithDerivative, Matrix,
    Vector,
};
use rustycad_tkernel::standard::Failure;
mod adapter;
mod results;
/// Scalar root result; `FunctionRoot::new(&mut function,3.0,1e-6)?`.
pub struct FunctionRoot {
    done: bool,
    root: f64,
    derivative: f64,
    value: f64,
    iterations: i32,
}
impl FunctionRoot {
    /// Uses the default 100-iteration limit; `FunctionRoot::new(&mut function,3.0,1e-6)?`.
    pub fn new(
        function: &mut impl FunctionWithDerivative,
        guess: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        Self::with_iterations(function, guess, tolerance, 100)
    }
    /// Configures the iteration limit without finite domain bounds; `FunctionRoot::with_iterations(&mut function,3.0,1e-6,50)?`.
    pub fn with_iterations(
        function: &mut impl FunctionWithDerivative,
        guess: f64,
        tolerance: f64,
        limit: i32,
    ) -> Result<Self, Failure> {
        Self::solve(function, guess, tolerance, None, limit)
    }
    /// Searches within explicit bounds; `FunctionRoot::bounded(&mut function,1.5,1e-6,1.0,3.0,100)?`.
    pub fn bounded(
        function: &mut impl FunctionWithDerivative,
        guess: f64,
        tolerance: f64,
        lower: f64,
        upper: f64,
        limit: i32,
    ) -> Result<Self, Failure> {
        Self::solve(function, guess, tolerance, Some([lower, upper]), limit)
    }
    fn solve(
        function: &mut impl FunctionWithDerivative,
        guess: f64,
        tolerance: f64,
        bounds: Option<[f64; 2]>,
        limit: i32,
    ) -> Result<Self, Failure> {
        let solver: FunctionSetRoot = adapter::search(function, guess, tolerance, bounds, limit)?;
        let mut result: Self = Self {
            done: solver.is_done(),
            root: 0.0,
            derivative: 0.0,
            value: 0.0,
            iterations: 0,
        };
        if !result.done {
            return Ok(result);
        }
        result.iterations = solver.iterations()?;
        function.state_number();
        result.root = solver.root()?[1];
        result.derivative = solver.derivative()?[(1, 1)];
        function.value(result.root, &mut result.value);
        Ok(result)
    }
}
impl FunctionRoot {
    /// Writes the native `Dump` text; `root.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl std::fmt::Write) -> std::fmt::Result {
        use crate::math::stream_real;
        write!(stream, "math_FunctionRoot ")?;
        if !self.done {
            return writeln!(stream, " Status = not Done ");
        }
        writeln!(stream, " Status = Done ")?;
        writeln!(stream, " Number of iterations = {}", self.iterations)?;
        writeln!(stream, " The Root is: {}", stream_real(self.root))?;
        writeln!(
            stream,
            "The value at the root is: {}",
            stream_real(self.value)
        )
    }
}
display_via_dump!(FunctionRoot);
