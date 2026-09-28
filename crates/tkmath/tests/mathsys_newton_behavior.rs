//! Generic Newton validation and native finalization callback regressions.
use rustycad_tkmath::{
    math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector},
    math_sys::*,
    math_utils::Status,
};
struct FinalizationFailure {
    calls: usize,
}
impl FunctionSet for FinalizationFailure {
    fn variables(&self) -> i32 {
        1
    }
    fn equations(&self) -> i32 {
        1
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        *f.change_at(0).unwrap() = *x.at(0).unwrap();
        true
    }
}
impl FunctionSetWithDerivatives for FinalizationFailure {
    fn derivatives(&mut self, _: &Vector<'_>, j: &mut Matrix<'_>) -> bool {
        *j.change_at(0, 0).unwrap() = 1.0;
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, j: &mut Matrix<'_>) -> bool {
        self.calls += 1;
        if self.calls == 2 {
            return false;
        }
        self.value(x, f) && self.derivatives(x, j)
    }
}
#[test]
fn converged_start_still_propagates_finalization_failure() {
    let mut f = FinalizationFailure { calls: 0 };
    let start = Vector::filled(7, 7, 0.0).unwrap();
    let result = newton_uniform(&mut f, &start, 1e-10, 1e-10, 10);
    assert_eq!(result.status, Status::CallbackError);
    assert_eq!(f.calls, 2);
    assert!(result.solution.is_some());
    assert!(result.jacobian.is_none());
    assert_eq!(result.iterations, 0);
    assert!(!bool::from(&result));
}
#[test]
fn invalid_tolerance_precedes_function_evaluation() {
    let mut f = FinalizationFailure { calls: 0 };
    let start = Vector::filled(0, 0, 1.0).unwrap();
    let result = newton_uniform(&mut f, &start, f64::NAN, 1e-10, 10);
    assert_eq!(result.status, Status::InvalidInput);
    assert_eq!(f.calls, 0);
    assert!(result.solution.is_none());
    assert!(!bool::from(&SystemResult::default()));
}
#[test]
fn safe_norm_does_not_overflow_finite_hypotenuse() {
    let vector = Vector::filled(-1, 0, 1e200).unwrap();
    assert_eq!(linear::safe_norm(&vector), 1e200_f64.hypot(1e200));
    assert!(linear::safe_norm(&vector).is_finite());
}
