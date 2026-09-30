use super::KronrodSingleIntegration;
use rustycad_tkernel::standard::Failure;
impl KronrodSingleIntegration {
    /// Reports successful quadrature evaluations; `integral.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reads the integral; `integral.value()?`.
    pub fn value(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.value)
    }
    /// Reads native error (absolute in single-rule mode); `integral.error_reached()?`.
    pub fn error_reached(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.error)
    }
    /// Reads the accumulated absolute error; `integral.absolute_error()?`.
    pub fn absolute_error(&self) -> Result<f64, Failure> {
        self.check_done()?;
        Ok(self.absolute)
    }
    /// Reads the odd Kronrod point count; `integral.order_reached()?`.
    pub fn order_reached(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.order)
    }
    /// Reads completed adaptive iterations; `integral.iterations_reached()?`.
    pub fn iterations_reached(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.iterations)
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                "math_KronrodSingleIntegration",
                "",
            ));
        }
        Ok(())
    }
}
