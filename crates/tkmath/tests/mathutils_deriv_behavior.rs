//! Supplemental MathUtils_Deriv.hxx tests at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkmath::math::{MultipleVarFunction, Vector};
use rustycad_tkmath::math_utils::*;
#[test]
fn scalar_derivatives_use_actual_spacing() {
    let mut linear = |x: f64, y: &mut f64| {
        *y = x;
        true
    };
    let mut derivative: f64 = 91.0;
    assert!(central_difference(
        &mut linear,
        1e100,
        &mut derivative,
        1e-8
    ));
    assert_eq!(derivative, 1.0);
    assert!(forward_difference(
        &mut linear,
        1e100,
        1e100,
        &mut derivative,
        1e-8
    ));
    assert_eq!(derivative, 1.0);
    let mut square = |x: f64, y: &mut f64| {
        *y = x * x;
        true
    };
    assert!(second_derivative(
        &mut square,
        2.0,
        4.0,
        &mut derivative,
        1e-5
    ));
    assert!((derivative - 2.0).abs() < 1e-5);
}
#[test]
fn scalar_failures_preserve_output_and_callback_order() {
    let mut calls: Vec<f64> = Vec::new();
    let mut failing = |x: f64, y: &mut f64| {
        calls.push(x);
        *y = x;
        x > 2.0
    };
    let mut derivative: f64 = 91.0;
    assert!(!central_difference(&mut failing, 2.0, &mut derivative, 0.5));
    assert_eq!(derivative, 91.0);
    assert_eq!(calls, vec![2.5, 1.5]);
    assert!(!forward_difference(
        &mut |_, _: &mut f64| true,
        f64::MAX,
        1.0,
        &mut derivative,
        1.0
    ));
    assert!(!second_derivative(
        &mut |_, _: &mut f64| true,
        2.0,
        f64::NAN,
        &mut derivative,
        1e-5
    ));
    assert_eq!(derivative, 91.0);
}
struct LinearObjective {
    fail: bool,
}
impl MultipleVarFunction for LinearObjective {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        *value = *point.at(0).unwrap();
        !self.fail
    }
}
#[test]
fn gradients_restore_input_and_support_custom_bounds() {
    let mut point = Vector::filled(-2, -1, 1e100).unwrap();
    let mut gradient = Vector::filled(3, 4, 91.0).unwrap();
    let mut function = LinearObjective { fail: false };
    assert!(numerical_gradient_adaptive(
        &mut function,
        &mut point,
        &mut gradient,
        1e-8
    ));
    assert_eq!(
        (*gradient.at(0).unwrap(), *gradient.at(1).unwrap()),
        (1.0, 0.0)
    );
    assert!(numerical_gradient(
        &mut function,
        &mut point,
        &mut gradient,
        1e-8
    ));
    function.fail = true;
    assert!(!numerical_gradient(
        &mut function,
        &mut point,
        &mut gradient,
        1e-8
    ));
    assert_eq!(
        (*point.at(0).unwrap(), *point.at(1).unwrap()),
        (1e100, 1e100)
    );
    assert!(!numerical_gradient(
        &mut function,
        &mut point,
        &mut Vector::with_size(1),
        1e-8
    ));
}

struct QuadraticObjective {
    calls: usize,
    fail_at: usize,
}
impl MultipleVarFunction for QuadraticObjective {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        self.calls += 1;
        let x: f64 = *point.at(0).unwrap();
        let y: f64 = *point.at(1).unwrap();
        *value = x * x + 3.0 * x * y + 2.0 * y * y;
        self.calls != self.fail_at
    }
}
#[test]
fn hessian_mixed_samples_and_failure_restoration() {
    for fail_at in 0..=9 {
        let mut point = Vector::filled(-1, 0, 2.0).unwrap();
        let mut hessian = rustycad_tkmath::math::Matrix::filled(3, 4, -4, -3, 91.0).unwrap();
        let mut objective = QuadraticObjective { calls: 0, fail_at };
        assert_eq!(
            numerical_hessian(&mut objective, &mut point, &mut hessian, 1e-3),
            fail_at == 0
        );
        assert_eq!((*point.at(0).unwrap(), *point.at(1).unwrap()), (2.0, 2.0));
        if fail_at == 0 {
            check_hessian(&hessian);
        }
    }
}
fn check_hessian(hessian: &rustycad_tkmath::math::Matrix<'_>) {
    assert!((hessian.at(0, 0).unwrap() - 2.0).abs() < 1e-7);
    assert!((hessian.at(1, 1).unwrap() - 4.0).abs() < 1e-7);
    assert!((hessian.at(0, 1).unwrap() - 3.0).abs() < 1e-7);
    assert_eq!(hessian.at(0, 1).unwrap(), hessian.at(1, 0).unwrap());
}
struct LinearSystem {
    calls: usize,
    fail_at: usize,
}
impl rustycad_tkmath::math::FunctionSet for LinearSystem {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        self.calls += 1;
        let x: f64 = *point.at(0).unwrap();
        let y: f64 = *point.at(1).unwrap();
        *residual.change_at(0).unwrap() = 2.0 * x + 3.0 * y;
        *residual.change_at(1).unwrap() = 4.0 * x + 5.0 * y;
        self.calls != self.fail_at
    }
}
#[test]
fn jacobian_custom_bounds_and_failure_restoration() {
    for fail_at in 0..=4 {
        let mut point = Vector::filled(-1, 0, 2.0).unwrap();
        let mut jacobian = rustycad_tkmath::math::Matrix::filled(3, 4, -4, -3, 91.0).unwrap();
        let mut system = LinearSystem { calls: 0, fail_at };
        assert_eq!(
            numerical_jacobian(&mut system, &mut point, &mut jacobian, 1e-3),
            fail_at == 0
        );
        assert_eq!((*point.at(0).unwrap(), *point.at(1).unwrap()), (2.0, 2.0));
        if fail_at == 0 {
            check_jacobian(&jacobian);
        }
    }
}
fn check_jacobian(jacobian: &rustycad_tkmath::math::Matrix<'_>) {
    assert!((jacobian.at(0, 0).unwrap() - 2.0).abs() < 1e-10);
    assert!((jacobian.at(0, 1).unwrap() - 3.0).abs() < 1e-10);
    assert!((jacobian.at(1, 0).unwrap() - 4.0).abs() < 1e-10);
    assert!((jacobian.at(1, 1).unwrap() - 5.0).abs() < 1e-10);
}
