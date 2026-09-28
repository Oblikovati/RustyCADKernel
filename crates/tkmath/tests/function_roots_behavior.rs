//! Sampling, callback states, and error guards retained from OCCT 3d097a0.
use rustycad_tkmath::math::{Function, FunctionRoots, FunctionWithDerivative};
struct StatefulLine {
    calls: usize,
    fail: usize,
    states: i32,
    constant: bool,
}
impl Function for StatefulLine {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        self.calls += 1;
        *value = if self.constant { 0.0 } else { x - 0.731 };
        self.calls != self.fail
    }
    fn state_number(&mut self) -> i32 {
        self.states += 1;
        self.states
    }
}
impl FunctionWithDerivative for StatefulLine {
    fn derivative(&mut self, _x: f64, derivative: &mut f64) -> bool {
        *derivative = 1.0;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *derivative = 1.0;
        self.value(x, value)
    }
}
#[test]
fn failed_sample_still_evaluates_every_sample_and_guards_all_results() {
    let mut f: StatefulLine = StatefulLine {
        calls: 0,
        fail: 3,
        states: 0,
        constant: false,
    };
    let s: FunctionRoots = FunctionRoots::new(&mut f, -1.0, 1.0, 0);
    assert_eq!(f.calls, 21);
    assert_eq!(f.states, 0);
    assert!(!s.is_done());
    assert_eq!(
        s.is_all_null().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        s.number_of_solutions().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(s.value(0).unwrap_err().exception_type(), "StdFail_NotDone");
    assert_eq!(
        s.state_number(0).unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
}
#[test]
fn duplicate_candidates_capture_one_state_and_use_sequence_bounds_errors() {
    let mut f: StatefulLine = StatefulLine {
        calls: 0,
        fail: 0,
        states: 0,
        constant: false,
    };
    let s: FunctionRoots = FunctionRoots::new(&mut f, 0.731, 2.0, -5);
    assert_eq!(s.number_of_solutions().unwrap(), 1);
    assert_eq!(s.value(1).unwrap(), 0.731);
    assert_eq!(s.state_number(1).unwrap(), 1);
    assert_eq!(f.states, 1);
    assert_eq!(
        s.value(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        s.state_number(2).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}
#[test]
fn all_null_short_circuits_root_and_state_callbacks() {
    let mut f: StatefulLine = StatefulLine {
        calls: 0,
        fail: 0,
        states: 0,
        constant: true,
    };
    let s: FunctionRoots = FunctionRoots::new(&mut f, -1.0, 1.0, 20);
    assert!(s.is_done());
    assert!(s.is_all_null().unwrap());
    assert_eq!(s.number_of_solutions().unwrap(), 0);
    assert_eq!(f.calls, 41);
    assert_eq!(f.states, 0);
}
