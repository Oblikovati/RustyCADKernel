use super::{Failure, NewtonFunctionRoot};
impl NewtonFunctionRoot {
    /// Reports native convergence; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reads the resulting scalar root; `solver.root()?`.
    pub fn root(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.root)
    }
    /// Reads the last combined callback's function output; `solver.value()?`.
    pub fn value(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.value)
    }
    /// Reads the Newton iteration's saved derivative; `solver.derivative()?`.
    pub fn derivative(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.derivative)
    }
    /// Reads the Newton iteration's convergence iteration; `solver.iterations()?`.
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
