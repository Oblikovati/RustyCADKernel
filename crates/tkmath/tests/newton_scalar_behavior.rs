//! Native best-sample restoration, exact-root derivative handling, and iteration accounting.
use rustycad_tkmath::math::{Function, FunctionWithDerivative, NewtonFunctionRoot};
struct ScriptedDerivative {
    calls: usize,
}
impl Function for ScriptedDerivative {
    fn value(&mut self, _x: f64, _value: &mut f64) -> bool {
        panic!("Newton must use the combined callback")
    }
}
impl FunctionWithDerivative for ScriptedDerivative {
    fn derivative(&mut self, _x: f64, _derivative: &mut f64) -> bool {
        panic!("Newton must use the combined callback")
    }
    fn values(&mut self, _x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        (*value, *derivative) = if self.calls == 0 {
            (1.0, 0.25)
        } else {
            (2.0, 4.0)
        };
        self.calls += 1;
        true
    }
}
#[test]
fn best_position_is_restored_but_value_and_derivative_are_the_last_evaluation() {
    let mut function: ScriptedDerivative = ScriptedDerivative { calls: 0 };
    let solver: NewtonFunctionRoot = NewtonFunctionRoot::new(&mut function, 5.0, 1.0, 3.0);
    assert!(solver.is_done());
    assert_eq!(function.calls, 2);
    assert_eq!(solver.root().unwrap(), 5.0);
    assert_eq!(solver.value().unwrap(), 2.0);
    assert_eq!(solver.derivative().unwrap(), 4.0);
    assert_eq!(solver.iterations().unwrap(), 3);
}
#[test]
fn final_increment_can_exceed_limit_after_an_acceptable_last_step() {
    let mut function: ScriptedDerivative = ScriptedDerivative { calls: 0 };
    let solver: NewtonFunctionRoot =
        NewtonFunctionRoot::with_iterations(&mut function, 5.0, 1.0, 3.0, 2);
    assert_eq!(function.calls, 2);
    assert!(!solver.is_done());
    assert!(solver.root().is_err());
    assert!(solver.value().is_err());
    assert!(solver.derivative().is_err());
    assert!(solver.iterations().is_err());
}
#[test]
fn maximal_tolerances_can_report_done_without_a_callback() {
    let mut function: ScriptedDerivative = ScriptedDerivative { calls: 0 };
    let solver: NewtonFunctionRoot =
        NewtonFunctionRoot::new(&mut function, 5.0, f64::MAX, f64::MAX);
    assert!(solver.is_done());
    assert_eq!(function.calls, 0);
    assert_eq!(solver.root().unwrap(), f64::MAX);
    assert_eq!(solver.value().unwrap(), f64::MAX);
    assert_eq!(solver.derivative().unwrap(), 0.0);
    assert_eq!(solver.iterations().unwrap(), 1);
}
