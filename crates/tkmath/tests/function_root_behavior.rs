//! Scalar adapter state capture and the final native Value callback.
use rustycad_tkmath::math::{Function, FunctionRoot, FunctionWithDerivative};
struct Stateful {
    states: i32,
    combined_success: bool,
}
impl Function for Stateful {
    fn value(&mut self, _x: f64, value: &mut f64) -> bool {
        *value = 91.0;
        false
    }
    fn state_number(&mut self) -> i32 {
        self.states += 1;
        self.states
    }
}
impl FunctionWithDerivative for Stateful {
    fn derivative(&mut self, _x: f64, derivative: &mut f64) -> bool {
        *derivative = 1.0;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = x - 2.0;
        *derivative = 1.0;
        self.combined_success
    }
}
#[test]
fn scalar_state_is_saved_once_and_final_value_status_is_ignored() {
    let mut function: Stateful = Stateful {
        states: 0,
        combined_success: true,
    };
    let result: FunctionRoot = FunctionRoot::new(&mut function, 2.0, 1e-8).unwrap();
    assert!(result.is_done());
    assert_eq!(result.iterations().unwrap(), 0);
    assert_eq!(result.root().unwrap(), 2.0);
    assert_eq!(result.derivative().unwrap(), 1.0);
    assert_eq!(result.value().unwrap(), 91.0);
    assert_eq!(function.states, 1);
}
#[test]
fn failed_adapter_does_not_save_scalar_state() {
    let mut function: Stateful = Stateful {
        states: 0,
        combined_success: false,
    };
    let result: FunctionRoot = FunctionRoot::new(&mut function, 2.0, 1e-8).unwrap();
    assert!(!result.is_done());
    assert_eq!(function.states, 0);
    assert!(result.root().is_err());
    assert!(result.value().is_err());
    assert!(result.derivative().is_err());
    assert!(result.iterations().is_err());
}
