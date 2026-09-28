//! Early convergence preserves cached values and failures reset a reusable solver.
use rustycad_tkmath::math::{BissecNewton, Function, FunctionWithDerivative};
struct Quadratic {
    success: bool,
}
impl Function for Quadratic {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = (x - 2.0) * (x - 2.0) - 1.0;
        self.success
    }
}
impl FunctionWithDerivative for Quadratic {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = 2.0 * (x - 2.0);
        self.success
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        self.value(x, value);
        self.derivative(x, derivative)
    }
}
#[test]
fn step_convergence_keeps_previous_function_and_derivative() {
    let mut function: Quadratic = Quadratic { success: true };
    let mut solver: BissecNewton = BissecNewton::new(1.0);
    solver.perform(&mut function, 0.0, 1.5, 100);
    assert!(solver.is_done());
    assert_eq!(solver.root().unwrap(), 0.975);
    assert_eq!(solver.value().unwrap(), 0.5625);
    assert_eq!(solver.derivative().unwrap(), -2.5);
}
#[test]
fn evaluation_failure_clears_a_previous_success() {
    let mut function: Quadratic = Quadratic { success: true };
    let mut solver: BissecNewton = BissecNewton::new(1e-10);
    solver.perform(&mut function, 0.0, 1.5, 100);
    assert!(solver.is_done());
    function.success = false;
    solver.perform(&mut function, 0.0, 1.5, 100);
    assert!(!solver.is_done());
    assert_eq!(
        solver.root().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        solver.value().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        solver.derivative().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
}
