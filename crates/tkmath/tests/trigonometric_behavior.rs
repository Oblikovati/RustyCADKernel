//! Native analytical multiplicities, one-period windows, and reusable status.
use rustycad_tkmath::math::{
    Function, FunctionWithDerivative, TrigonometricEquation, TrigonometricRoots,
};
use std::f64::consts::PI;
#[test]
fn cosine_factor_at_pi_retains_its_double_root() {
    let solver: TrigonometricRoots =
        TrigonometricRoots::new(1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 2.0 * PI);
    assert_eq!(solver.solutions().unwrap(), 4);
    assert_eq!(solver.value(2).unwrap(), PI);
    assert_eq!(solver.value(3).unwrap(), PI);
}
#[test]
fn analytical_sine_preserves_coincident_roots_and_limits_search_to_one_period() {
    let solver: TrigonometricRoots = TrigonometricRoots::sine(1.0, -1.0, 0.0, 8.0 * PI);
    assert_eq!(solver.solutions().unwrap(), 2);
    assert_eq!(solver.value(1).unwrap(), PI / 2.0);
    assert_eq!(solver.value(2).unwrap(), PI / 2.0);
}
#[test]
fn perform_resets_infinite_status_and_checks_root_range() {
    let mut solver: TrigonometricRoots = TrigonometricRoots::sine(0.0, 0.0, 0.0, 2.0 * PI);
    assert_eq!(
        solver.value(0).unwrap_err().exception_type(),
        "StdFail_InfiniteSolutions"
    );
    assert_eq!(
        solver.solutions().unwrap_err().exception_type(),
        "StdFail_InfiniteSolutions"
    );
    solver.perform([0.0, 0.0, 0.0, 1.0, 0.0], [0.0, 2.0 * PI]);
    assert!(!solver.infinite_roots());
    assert_eq!(solver.solutions().unwrap(), 2);
    assert_eq!(
        solver.value(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        solver.value(3).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}
#[test]
fn equation_callbacks_use_native_value_and_derivative_protocols() {
    let mut function: TrigonometricEquation = TrigonometricEquation::new(1.0, 2.0, 3.0, 4.0, 5.0);
    let (mut value, mut derivative): (f64, f64) = (0.0, 0.0);
    assert!(function.value(0.0, &mut value));
    assert_eq!(value, 9.0);
    assert!(function.derivative(0.0, &mut derivative));
    assert_eq!(derivative, 8.0);
    assert!(function.values(0.0, &mut value, &mut derivative));
    assert_eq!((value, derivative), (9.0, 8.0));
    assert_eq!(function.state_number(), 0);
}
