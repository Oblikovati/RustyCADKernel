//! Native stationary-point and divergence behavior beyond the legacy declarations.
use rustycad_tkmath::math::{
    FunctionSet, FunctionSetRoot, FunctionSetWithDerivatives, Matrix, Vector,
};
struct Constant {
    evaluations: i32,
    states: i32,
}
impl FunctionSet for Constant {
    fn variables(&self) -> i32 {
        1
    }
    fn equations(&self) -> i32 {
        1
    }
    fn value(&mut self, _point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        self.evaluations += 1;
        residual[1] = 1.0;
        true
    }
    fn state_number(&mut self) -> i32 {
        self.states += 1;
        43
    }
}
impl FunctionSetWithDerivatives for Constant {
    fn derivatives(&mut self, _point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        jacobian[(1, 1)] = 0.0;
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
fn zero_gradient_reports_success_even_with_nonzero_residual() {
    let mut function: Constant = Constant {
        evaluations: 0,
        states: 0,
    };
    let tolerance: Vector<'static> = Vector::filled(1, 1, 1e-8).unwrap();
    let start: Vector<'static> = Vector::filled(1, 1, 2.0).unwrap();
    let mut solver: FunctionSetRoot = FunctionSetRoot::new(&function, &tolerance, 100).unwrap();
    solver.perform(&mut function, &start, false).unwrap();
    assert!(solver.is_done());
    assert_eq!(solver.iterations().unwrap(), 0);
    assert_eq!(solver.root().unwrap()[1], 2.0);
    assert_eq!(solver.state_number().unwrap(), 43);
    assert_eq!(function.evaluations, 1);
    assert_eq!(function.states, 1);
    // Native Delta is uninitialized on this early-success path; no numeric claim is made for it.
}
#[test]
fn stop_on_divergence_precedes_callbacks_and_state_capture() {
    let mut function: Constant = Constant {
        evaluations: 0,
        states: 0,
    };
    let tolerance: Vector<'static> = Vector::filled(1, 1, 1e-8).unwrap();
    let start: Vector<'static> = Vector::filled(1, 1, 2.0).unwrap();
    let bound: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    let mut solver: FunctionSetRoot = FunctionSetRoot::new(&function, &tolerance, 100).unwrap();
    solver
        .perform_bounded(&mut function, &start, &bound, &bound, true)
        .unwrap();
    assert!(!solver.is_done());
    assert!(solver.is_divergent());
    assert_eq!(function.evaluations, 0);
    assert_eq!(function.states, 0);
    assert!(solver.root().is_err());
    assert!(solver.state_number().is_err());
}
#[test]
fn failed_output_copies_preserve_existing_storage() {
    let function: Constant = Constant {
        evaluations: 0,
        states: 0,
    };
    let solver: FunctionSetRoot = FunctionSetRoot::without_tolerance(&function, 100).unwrap();
    let mut vector: Vector<'static> = Vector::filled(1, 1, 91.0).unwrap();
    let mut matrix: Matrix<'static> = Matrix::filled(1, 1, 1, 1, 91.0).unwrap();
    assert!(solver.copy_root(&mut vector).is_err());
    assert!(solver.copy_derivative(&mut matrix).is_err());
    assert!(solver.copy_function_set_errors(&mut vector).is_err());
    assert!(solver.iterations().is_err());
    assert_eq!(vector[1], 91.0);
    assert_eq!(matrix[(1, 1)], 91.0);
}
