use super::{Failure, Matrix, Uzawa, Vector};
impl Uzawa {
    /// Reports native completion; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Borrows the projected point after success; `solver.value()?`.
    pub fn value(&self) -> Result<&Vector<'static>, Failure> {
        self.check_done()?;
        Ok(&self.solution)
    }
    /// Borrows initial residual storage, whose row-count prefix is defined; `solver.initial_error()?`.
    pub fn initial_error(&self) -> Result<&Vector<'static>, Failure> {
        self.check_done()?;
        Ok(&self.initial)
    }
    /// Borrows the displacement from the starting point; `solver.error()?`.
    pub fn error(&self) -> Result<&Vector<'static>, Failure> {
        self.check_done()?;
        Ok(&self.error)
    }
    /// Copies dual variables without a Done guard, as native Duale does; `solver.copy_dual(&mut output)?`.
    pub fn copy_dual(&self, output: &mut Vector<'_>) -> Result<(), Failure> {
        output.initialized(&self.dual)
    }
    /// Reads the native iteration counter; `solver.iterations()?`.
    pub fn iterations(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.iterations)
    }
    /// Borrows native inverse storage; only the lower triangle after an equality solve is defined; `solver.inverse_constraints()?`.
    pub fn inverse_constraints(&self) -> Result<&Matrix<'static>, Failure> {
        self.check_done()?;
        Ok(&self.inverse)
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
