//! Brent bracketing with OCCT's exact update sequence and stopping conditions.
use super::Function;
use rustycad_tkernel::standard::Failure;
mod iterate;
/// Result of bracketed root finding; `BracketedRoot::new(&mut function,0.0,2.0,1e-10)`.
pub struct BracketedRoot {
    root: f64,
    value: f64,
    iterations: i32,
    done: bool,
}
struct Bracket {
    a: f64,
    fa: f64,
    c: f64,
    fc: f64,
    d: f64,
    e: f64,
}
impl BracketedRoot {
    /// Uses 100 iterations and native ZEPS=1e-12; `BracketedRoot::new(&mut f,0.0,2.0,1e-10)`.
    pub fn new(function: &mut impl Function, first: f64, second: f64, tolerance: f64) -> Self {
        Self::with_options(function, first, second, tolerance, 100, 1e-12)
    }
    /// Preserves native iteration limits and relative tolerance; `BracketedRoot::with_options(&mut f,0.0,2.0,1e-10,200,1e-15)`.
    pub fn with_options(
        function: &mut impl Function,
        first: f64,
        second: f64,
        tolerance: f64,
        limit: i32,
        epsilon: f64,
    ) -> Self {
        let mut solver: Self = Self {
            root: second,
            value: 0.0,
            iterations: 0,
            done: false,
        };
        let mut bracket: Bracket = Bracket {
            a: first,
            fa: 0.0,
            c: 0.0,
            fc: 0.0,
            d: 0.0,
            e: 0.0,
        };
        function.value(first, &mut bracket.fa);
        function.value(second, &mut solver.value);
        if bracket.fa * solver.value > 0.0 {
            return solver;
        }
        bracket.fc = solver.value;
        solver.iterate(function, &mut bracket, tolerance, limit, epsilon);
        solver
    }
    /// Reports native convergence; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reads the root after convergence; `solver.root()?`.
    pub fn root(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.root)
    }
    /// Reads the last function output; `solver.value()?`.
    pub fn value(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.value)
    }
    /// Reads the one-based convergence iteration; `solver.iterations()?`.
    pub fn iterations(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.iterations)
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
impl BracketedRoot {
    /// Writes the native `Dump` text; `root.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl std::fmt::Write) -> std::fmt::Result {
        use crate::math::stream_real;
        write!(stream, "math_BracketedRoot ")?;
        if !self.done {
            return writeln!(stream, " Status = not Done ");
        }
        writeln!(stream, " Status = Done ")?;
        writeln!(stream, " Number of iterations = {}", self.iterations)?;
        writeln!(stream, " The Root is: {}", stream_real(self.root))?;
        writeln!(
            stream,
            " The value at the root is: {}",
            stream_real(self.value)
        )
    }
}
display_via_dump!(BracketedRoot);
