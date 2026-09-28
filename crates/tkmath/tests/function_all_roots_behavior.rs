//! Custom sampling, native numeric failures, and endpoint state capture.
use rustycad_tkmath::math::{
    Function, FunctionAllRoots, FunctionSample, FunctionSampling, FunctionWithDerivative,
};
struct RecordedLine {
    calls: usize,
    fail: usize,
    states: i32,
    zero: bool,
}
impl Function for RecordedLine {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        self.calls += 1;
        *value = if self.zero { 0.0 } else { x };
        self.calls != self.fail
    }
    fn state_number(&mut self) -> i32 {
        self.states += 1;
        self.states
    }
}
impl FunctionWithDerivative for RecordedLine {
    fn derivative(&mut self, _x: f64, derivative: &mut f64) -> bool {
        *derivative = 1.0;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *derivative = 1.0;
        self.value(x, value)
    }
}
struct NonuniformSample;
impl FunctionSampling for NonuniformSample {
    fn bounds(&self) -> [f64; 2] {
        [-2.0, 2.0]
    }
    fn number_of_points(&self) -> i32 {
        3
    }
    fn parameter(&self, index: i32) -> f64 {
        [-2.0, -1.0, 2.0][(index - 1) as usize]
    }
}
#[test]
fn custom_parameter_sampling_preserves_full_null_interval_states() {
    let mut f: RecordedLine = RecordedLine {
        calls: 0,
        fail: 0,
        states: 0,
        zero: true,
    };
    let s: FunctionAllRoots = FunctionAllRoots::new(&mut f, &NonuniformSample, [1e-6; 3]).unwrap();
    assert!(s.is_done());
    assert_eq!(s.number_of_intervals().unwrap(), 1);
    assert_eq!(s.number_of_points().unwrap(), 0);
    assert_eq!(s.interval(1).unwrap(), [-2.0, 2.0]);
    assert_eq!(s.interval_state(1).unwrap(), [1, 2]);
    assert_eq!((f.calls, f.states), (5, 2));
    assert_eq!(
        s.interval(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        s.point_state(1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}
#[test]
fn failure_during_isolated_search_becomes_numeric_error() {
    let mut f: RecordedLine = RecordedLine {
        calls: 0,
        fail: 23,
        states: 0,
        zero: false,
    };
    let sample: FunctionSample = FunctionSample::new(-2.0, 2.0, 11);
    let result: Result<FunctionAllRoots, _> = FunctionAllRoots::new(&mut f, &sample, [1e-6; 3]);
    assert_eq!(
        result.err().unwrap().exception_type(),
        "Standard_NumericError"
    );
    assert_eq!(f.calls, 34);
    assert_eq!(f.states, 0);
}
