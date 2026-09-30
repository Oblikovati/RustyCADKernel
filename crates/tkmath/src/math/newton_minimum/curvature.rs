use super::{Failure, Gauss, Jacobi, NewtonMinimum, Status, Vector};
impl NewtonMinimum {
    pub(super) fn adjust_curvature(&mut self) -> Result<bool, Failure> {
        let solver: Jacobi = Jacobi::new(&self.hessian)?;
        if !solver.is_done() {
            return Ok(self.fail(Status::FunctionError));
        }
        let values: &Vector<'static> = solver.values()?;
        self.min_eigenvalue = min_eigenvalue(values)?;
        if self.min_eigenvalue.partial_cmp(&self.convexity) != Some(std::cmp::Ordering::Less) {
            return Ok(true);
        }
        self.convex = false;
        if !self.singularity
            || self.min_eigenvalue.abs().partial_cmp(&self.convexity)
                != Some(std::cmp::Ordering::Greater)
        {
            return Ok(self.fail(Status::FunctionError));
        }
        let shift: f64 = self.convexity + 0.1 * self.min_eigenvalue.abs() - self.min_eigenvalue;
        for index in 1..=self.result.gradient.length() {
            self.hessian[(index, index)] += shift;
        }
        Ok(true)
    }
    pub(super) fn solve_step(&mut self) -> Result<bool, Failure> {
        let solver: Gauss = Gauss::with_min_pivot(&self.hessian, self.convexity / 100.0)?;
        if !solver.is_done() {
            return Ok(self.fail(Status::DirectionSearchError));
        }
        solver.solve_into(&self.result.gradient, &mut self.step)?;
        Ok(true)
    }
}
// Native `Values()(Values().Min())`: Min() is 0 for no variable, and math_NewtonMinimum.cxx
// compiles the range check out, so it reads before the empty eigenvalue vector
// (undefined behaviour). The port raises the Standard_OutOfRange of a checked build.
fn min_eigenvalue(values: &Vector<'_>) -> Result<f64, Failure> {
    values.value(values.min_index()).copied().map_err(|_| {
        Failure::out_of_range(&format!(
            "math_NewtonMinimum: minimum eigenvalue index {} of {} variables; expected at least one variable",
            values.min_index(),
            values.length()
        ))
    })
}
