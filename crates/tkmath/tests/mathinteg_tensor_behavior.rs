//! Supplemental multidimensional validation and callback sequencing regressions.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::{
    math::{FunctionSet, MultipleVarFunction, Vector},
    math_integ::*,
    math_utils::Status,
};
struct ConstantTensor;
impl MultipleVarFunction for ConstantTensor {
    fn variables(&self) -> i32 {
        panic!("dimension callback must remain unused")
    }
    fn value(&mut self, _: &Vector<'_>, out: &mut f64) -> bool {
        *out = 1.0;
        true
    }
}
#[test]
fn tensor_overflow_is_rejected_before_quadrature_allocation() {
    let mut f: ConstantTensor = ConstantTensor;
    let lower: Vector = Vector::filled(0, 63, 0.0).unwrap();
    let upper: Vector = Vector::filled(0, 63, 1.0).unwrap();
    let mut orders: Array1<i32> = Array1::with_size(64);
    orders.init(61);
    let result = gauss_multiple(
        &mut f,
        64,
        &lower,
        &upper,
        &orders,
        &MultipleConfig::default(),
    );
    assert_eq!(result.status, Status::InvalidInput);
    assert_eq!(result.number_of_points, 0);
}
#[test]
fn tensor_reversed_axis_keeps_signed_volume() {
    let mut f: ConstantTensor = ConstantTensor;
    let mut lower: Vector = Vector::filled(0, 1, 0.0).unwrap();
    let upper: Vector = Vector::filled(0, 1, 1.0).unwrap();
    lower[0] = 2.0;
    let result = gauss_multiple_uniform(&mut f, 2, &lower, &upper, 3);
    assert!(result.is_done());
    assert!((result.value.unwrap() + 1.0).abs() < 1e-14);
    assert_eq!(result.number_of_points, 9);
}
struct NonFinitePair {
    calls: usize,
}
impl FunctionSet for NonFinitePair {
    fn variables(&self) -> i32 {
        panic!("variable count must remain unused")
    }
    fn equations(&self) -> i32 {
        1
    }
    fn value(&mut self, _: &Vector<'_>, out: &mut Vector<'_>) -> bool {
        self.calls += 1;
        *out.change_at(0).unwrap() = if self.calls == 1 { f64::INFINITY } else { 1.0 };
        true
    }
}
#[test]
fn set_finishes_symmetric_pair_before_nonfinite_check() {
    let mut f = NonFinitePair { calls: 0 };
    let result = gauss_set(&mut f, 0.0, 1.0, 2);
    assert_eq!(result.status, Status::NumericalError);
    assert_eq!(result.nb_points, 2);
    assert_eq!(f.calls, 2);
    assert!(!bool::from(&result));
    assert!(!bool::from(&SetResult::default()));
}
#[test]
fn zero_width_set_preserves_equation_count_and_zero_values() {
    let mut f = NonFinitePair { calls: 0 };
    let result = gauss_set(&mut f, 1.0, 1.0, 100);
    assert!(bool::from(&result));
    assert_eq!(result.nb_equations, 1);
    assert_eq!(*result.values.unwrap().at(0).unwrap(), 0.0);
    assert_eq!(f.calls, 0);
}
