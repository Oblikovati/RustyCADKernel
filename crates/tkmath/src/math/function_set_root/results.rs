use super::{Failure, FunctionSetRoot, Matrix, Vector};
impl FunctionSetRoot {
    /// Reports native convergence; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reports any attempted point outside the supplied bounds; `solver.is_divergent()`.
    pub fn is_divergent(&self) -> bool {
        self.divergent
    }
    /// Reads the converged root; `solver.root()?`.
    pub fn root(&self) -> Result<&Vector<'static>, Failure> {
        self.check_done()?;
        Ok(&self.sol)
    }
    /// Copies the root preserving destination bounds; `solver.copy_root(&mut output)?`.
    pub fn copy_root(&self, output: &mut Vector<'_>) -> Result<(), Failure> {
        output.initialized(self.root()?)
    }
    /// Reads the last derivative callback output; `solver.derivative()?`.
    pub fn derivative(&self) -> Result<&Matrix<'static>, Failure> {
        self.check_done()?;
        Ok(&self.jacobian)
    }
    /// Copies the derivative matrix; `solver.copy_derivative(&mut output)?`.
    pub fn copy_derivative(&self, output: &mut Matrix<'_>) -> Result<(), Failure> {
        output.initialized(self.derivative()?)
    }
    /// Reads the native last step vector; `solver.function_set_errors()?`.
    pub fn function_set_errors(&self) -> Result<&Vector<'static>, Failure> {
        self.check_done()?;
        Ok(&self.delta)
    }
    /// Copies the last step vector; `solver.copy_function_set_errors(&mut output)?`.
    pub fn copy_function_set_errors(&self, output: &mut Vector<'_>) -> Result<(), Failure> {
        output.initialized(self.function_set_errors()?)
    }
    /// Reads the number of iterations; `solver.iterations()?`.
    pub fn iterations(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.iteration)
    }
    /// Reads the saved callback state number; `solver.state_number()?`.
    pub fn state_number(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.state)
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
