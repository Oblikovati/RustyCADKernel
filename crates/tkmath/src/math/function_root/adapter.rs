use super::{
    Failure, FunctionSet, FunctionSetRoot, FunctionSetWithDerivatives, FunctionWithDerivative,
    Matrix, Vector,
};
struct ScalarSystem<'a, F> {
    function: &'a mut F,
}
impl<F: FunctionWithDerivative> FunctionSet for ScalarSystem<'_, F> {
    fn variables(&self) -> i32 {
        1
    }
    fn equations(&self) -> i32 {
        1
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        self.function.value(point[1], &mut residual[1])
    }
}
impl<F: FunctionWithDerivative> FunctionSetWithDerivatives for ScalarSystem<'_, F> {
    fn derivatives(&mut self, point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        self.function.derivative(point[1], &mut jacobian[(1, 1)])
    }
    fn values(
        &mut self,
        point: &Vector<'_>,
        residual: &mut Vector<'_>,
        jacobian: &mut Matrix<'_>,
    ) -> bool {
        self.function
            .values(point[1], &mut residual[1], &mut jacobian[(1, 1)])
    }
}
pub(super) fn search(
    function: &mut impl FunctionWithDerivative,
    guess: f64,
    tolerance: f64,
    bounds: Option<[f64; 2]>,
    limit: i32,
) -> Result<FunctionSetRoot, Failure> {
    let mut wrapped: ScalarSystem<'_, _> = ScalarSystem { function };
    let start: Vector<'static> = Vector::filled(1, 1, guess)?;
    let tolerances: Vector<'static> = Vector::filled(1, 1, tolerance)?;
    let mut solver: FunctionSetRoot = FunctionSetRoot::new(&wrapped, &tolerances, limit)?;
    if let Some([lower, upper]) = bounds {
        solver.perform_bounded(
            &mut wrapped,
            &start,
            &Vector::filled(1, 1, lower)?,
            &Vector::filled(1, 1, upper)?,
            false,
        )?;
    } else {
        solver.perform(&mut wrapped, &start, false)?;
    }
    Ok(solver)
}
