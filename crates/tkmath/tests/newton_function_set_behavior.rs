//! Newton result meaning, saved states, and singular-result guards.
use rustycad_tkmath::math::{
    FunctionSet, FunctionSetWithDerivatives, Matrix, NewtonFunctionSetRoot, Vector,
};
struct Linear {
    states: i32,
    slope: f64,
}
impl FunctionSet for Linear {
    fn variables(&self) -> i32 {
        1
    }
    fn equations(&self) -> i32 {
        1
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        residual[1] = self.slope * point[1] - 4.0;
        true
    }
    fn state_number(&mut self) -> i32 {
        self.states += 1;
        43
    }
}
impl FunctionSetWithDerivatives for Linear {
    fn derivatives(&mut self, _point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        jacobian[(1, 1)] = self.slope;
        true
    }
    fn values(
        &mut self,
        point: &Vector<'_>,
        residual: &mut Vector<'_>,
        jacobian: &mut Matrix<'_>,
    ) -> bool {
        self.value(point, residual);
        self.derivatives(point, jacobian)
    }
}
#[test]
fn function_set_errors_returns_step_and_saves_state_only_on_success() {
    let mut function: Linear = Linear {
        states: 0,
        slope: 2.0,
    };
    let tolerance: Vector<'static> = Vector::filled(1, 1, 10.0).unwrap();
    let start: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    let mut solver: NewtonFunctionSetRoot =
        NewtonFunctionSetRoot::new(&function, &tolerance, 1e-10, 100).unwrap();
    solver.perform(&mut function, &start).unwrap();
    assert_eq!(solver.iterations().unwrap(), 1);
    assert_eq!(solver.root().unwrap()[1], 2.0);
    assert_eq!(solver.function_set_errors().unwrap()[1], 2.0);
    assert_eq!(function.states, 1);
    function.slope = 0.0;
    solver.perform(&mut function, &start).unwrap();
    assert!(!solver.is_done());
    assert_eq!(function.states, 1);
    assert_eq!(
        solver.root().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
}
#[test]
fn unfinished_output_copies_preserve_destination() {
    let function: Linear = Linear {
        states: 0,
        slope: 2.0,
    };
    let solver: NewtonFunctionSetRoot =
        NewtonFunctionSetRoot::without_step_tolerance(&function, 1e-10, 100).unwrap();
    let mut vector: Vector<'static> = Vector::filled(5, 5, 91.0).unwrap();
    let mut matrix: Matrix<'static> = Matrix::filled(5, 5, 5, 5, 91.0).unwrap();
    assert!(solver.copy_root(&mut vector).is_err());
    assert!(solver.copy_function_set_errors(&mut vector).is_err());
    assert!(solver.copy_derivative(&mut matrix).is_err());
    assert!(solver.iterations().is_err());
    assert_eq!(vector[5], 91.0);
    assert_eq!(matrix[(5, 5)], 91.0);
}
