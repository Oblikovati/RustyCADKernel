use super::{Failure, TrigonometricRoots};
impl TrigonometricRoots {
    /// Reports native completion; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reports an identity equation; `solver.infinite_roots()`.
    pub fn infinite_roots(&self) -> bool {
        self.infinite
    }
    /// Counts roots with native multiplicity; `solver.solutions()?`.
    pub fn solutions(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.count)
    }
    /// Reads one-based native root order; `solver.value(1)?`.
    pub fn value(&self, index: i32) -> Result<f64, Failure> {
        self.check_done()?;
        if index < 1 || index > self.count {
            return Err(Failure::with_type("Standard_OutOfRange", " ", ""));
        }
        Ok(self.roots[(index - 1) as usize])
    }
    fn check_done(&self) -> Result<(), Failure> {
        if self.infinite {
            return Err(Failure::with_type("StdFail_InfiniteSolutions", " ", ""));
        }
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
