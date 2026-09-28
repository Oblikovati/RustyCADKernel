//! Supplemental MathRoot_Newton.hxx tests at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkmath::math::{Function, FunctionWithDerivative};
use rustycad_tkmath::math_root::{newton, newton_bounded};
use rustycad_tkmath::math_utils::{Config, Status};
struct SquareRoot {
    calls: usize,
    fail_at: usize,
}
impl Function for SquareRoot {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = x * x - 2.0;
        true
    }
}
impl FunctionWithDerivative for SquareRoot {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = 2.0 * x;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        self.calls += 1;
        self.value(x, value);
        self.derivative(x, derivative);
        self.calls != self.fail_at
    }
}
#[test]
fn newton_and_reversed_bounded_newton_converge() {
    let mut f = SquareRoot {
        calls: 0,
        fail_at: 0,
    };
    let result = newton(&mut f, 1.0, &Config::default());
    assert_eq!(result.status, Status::Ok);
    assert!((result.root.unwrap() - 2.0_f64.sqrt()).abs() < 1e-10);
    assert_eq!(result.derivative.unwrap(), 2.0 * result.root.unwrap());
    let result = newton_bounded(&mut f, 1.0, 2.0, 0.0, &Config::default());
    assert_eq!(result.status, Status::Ok);
    assert!((result.root.unwrap() - 2.0_f64.sqrt()).abs() < 1e-10);
}
#[test]
fn failure_retains_only_defined_native_fields() {
    let result = newton(
        &mut SquareRoot {
            calls: 0,
            fail_at: 2,
        },
        1.0,
        &Config::default(),
    );
    assert_eq!(result.status, Status::CallbackError);
    assert_eq!(result.iterations, 2);
    assert_eq!(result.root, Some(1.5));
    assert!(result.value.is_none());
    assert!(result.derivative.is_none());
    let result = newton_bounded(
        &mut SquareRoot {
            calls: 0,
            fail_at: 2,
        },
        1.0,
        0.0,
        2.0,
        &Config::default(),
    );
    assert_eq!(result.status, Status::CallbackError);
    assert_eq!(result.iterations, 0);
    assert!(result.root.is_none());
}
#[test]
fn limited_iteration_and_zero_derivative_results_are_coherent() {
    let config = Config {
        max_iterations: 1,
        ..Default::default()
    };
    let result = newton(
        &mut SquareRoot {
            calls: 0,
            fail_at: 0,
        },
        1.0,
        &config,
    );
    assert_eq!(result.status, Status::MaxIterations);
    assert_eq!(result.root, Some(1.0));
    assert_eq!(result.value, Some(-1.0));
    let result = newton(
        &mut SquareRoot {
            calls: 0,
            fail_at: 0,
        },
        0.0,
        &Config::default(),
    );
    assert_eq!(result.status, Status::NumericalError);
    assert_eq!(result.derivative, Some(0.0));
    let config = Config {
        x_tolerance: 0.0,
        ..Default::default()
    };
    assert_eq!(
        newton(
            &mut SquareRoot {
                calls: 0,
                fail_at: 0
            },
            1.0,
            &config
        )
        .status,
        Status::InvalidInput
    );
}
