//! Direct math_FunctionAllRoots_Test.cxx translation at OCCT 3d097a0.
#![allow(non_snake_case)]
use rustycad_tkmath::math::{Function, FunctionAllRoots, FunctionSample, FunctionWithDerivative};
struct CubicFunction;
impl Function for CubicFunction {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let _ = x;
        *value = x * x * x - 6.0 * x * x + 11.0 * x - 6.0;
        true
    }
}
impl FunctionWithDerivative for CubicFunction {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let _ = x;
        *derivative = 3.0 * x * x - 12.0 * x + 11.0;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let _ = x;
        *value = x * x * x - 6.0 * x * x + 11.0 * x - 6.0;
        *derivative = 3.0 * x * x - 12.0 * x + 11.0;
        true
    }
}
struct SineFunction;
impl Function for SineFunction {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let _ = x;
        *value = x.sin();
        true
    }
}
impl FunctionWithDerivative for SineFunction {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let _ = x;
        *derivative = x.cos();
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let _ = x;
        *value = x.sin();
        *derivative = x.cos();
        true
    }
}
struct NullIntervalFunction;
impl Function for NullIntervalFunction {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let _ = x;
        *value = if (2.0..=4.0).contains(&x) {
            0.0
        } else if x < 2.0 {
            x - 2.0
        } else {
            x - 4.0
        };
        true
    }
}
impl FunctionWithDerivative for NullIntervalFunction {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let _ = x;
        *derivative = if (2.0..=4.0).contains(&x) { 0.0 } else { 1.0 };
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let _ = x;
        *value = if (2.0..=4.0).contains(&x) {
            0.0
        } else if x < 2.0 {
            x - 2.0
        } else {
            x - 4.0
        };
        *derivative = if (2.0..=4.0).contains(&x) { 0.0 } else { 1.0 };
        true
    }
}
struct QuadraticFunction;
impl Function for QuadraticFunction {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let _ = x;
        *value = (x - 1.5) * (x - 1.5) - 0.25;
        true
    }
}
impl FunctionWithDerivative for QuadraticFunction {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let _ = x;
        *derivative = 2.0 * (x - 1.5);
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let _ = x;
        *value = (x - 1.5) * (x - 1.5) - 0.25;
        *derivative = 2.0 * (x - 1.5);
        true
    }
}
struct ZeroFunction;
impl Function for ZeroFunction {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let _ = x;
        *value = 0.0;
        true
    }
}
impl FunctionWithDerivative for ZeroFunction {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let _ = x;
        *derivative = 0.0;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let _ = x;
        *value = 0.0;
        *derivative = 0.0;
        true
    }
}
fn near(a: f64, b: f64, tolerance: f64) {
    assert!((a - b).abs() <= tolerance, "{a} != {b}");
}
fn near_any(root: f64, expected: &[f64]) -> bool {
    expected.iter().any(|v| (root - v).abs() < 1e-4)
}
#[test]
fn math_FunctionAllRoots__CubicFunctionBasic() {
    let mut f = CubicFunction;
    let sample = FunctionSample::new(0.0, 5.0, 21);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-6, 1e-6, 1e-8]).unwrap();
    assert!(s.is_done());
    assert!(s.number_of_points().unwrap() >= 2);
    if s.number_of_points().unwrap() >= 2 {
        assert!(near_any(s.point(1).unwrap(), &[1.0, 2.0, 3.0]));
        assert!(near_any(s.point(2).unwrap(), &[1.0, 2.0, 3.0]));
    }
}
#[test]
fn math_FunctionAllRoots__SineFunctionRoots() {
    let mut f = SineFunction;
    let sample = FunctionSample::new(-1.0, 7.0, 41);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-6, 1e-6, 1e-8]).unwrap();
    assert!(s.is_done());
    assert!(s.number_of_points().unwrap() >= 2);
    if s.number_of_points().unwrap() >= 1 {
        assert!(near_any(
            s.point(1).unwrap(),
            &[0.0, std::f64::consts::PI, 2.0 * std::f64::consts::PI]
        ));
    }
}
#[test]
fn math_FunctionAllRoots__NullIntervalFunction() {
    let mut f = NullIntervalFunction;
    let sample = FunctionSample::new(0.0, 6.0, 31);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-6, 1e-6, 1e-6]).unwrap();
    assert!(s.is_done());
    assert!(s.number_of_intervals().unwrap() >= 1);
    if s.number_of_intervals().unwrap() >= 1 {
        let [a, b] = s.interval(1).unwrap();
        near(a, 2.0, 1e-3);
        near(b, 4.0, 1e-3);
    }
}
#[test]
fn math_FunctionAllRoots__QuadraticTwoRoots() {
    let mut f = QuadraticFunction;
    let sample = FunctionSample::new(0.0, 3.0, 16);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-6, 1e-6, 1e-8]).unwrap();
    assert!(s.is_done());
    assert!(s.number_of_points().unwrap() >= 1);
    if s.number_of_points().unwrap() >= 1 {
        assert!(near_any(s.point(1).unwrap(), &[1.0, 2.0]));
    }
}
#[test]
fn math_FunctionAllRoots__ZeroFunction() {
    let mut f = ZeroFunction;
    let sample = FunctionSample::new(0.0, 1.0, 11);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-6, 1e-6, 1e-6]).unwrap();
    assert!(s.is_done());
    assert!(s.number_of_intervals().unwrap() >= 1);
    if s.number_of_intervals().unwrap() >= 1 {
        let [a, b] = s.interval(1).unwrap();
        near(a, 0.0, 1e-3);
        near(b, 1.0, 1e-3);
    }
}
#[test]
fn math_FunctionAllRoots__GetIntervalState() {
    let mut f = NullIntervalFunction;
    let sample = FunctionSample::new(0.0, 6.0, 31);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-6, 1e-6, 1e-6]).unwrap();
    assert!(s.is_done());
    if s.number_of_intervals().unwrap() >= 1 {
        let [a, b] = s.interval_state(1).unwrap();
        assert!(a >= 0);
        assert!(b >= 0);
        assert!(b >= a);
    }
}
#[test]
fn math_FunctionAllRoots__GetPointState() {
    let mut f = CubicFunction;
    let sample = FunctionSample::new(0.0, 5.0, 21);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-6, 1e-6, 1e-8]).unwrap();
    assert!(s.is_done());
    if s.number_of_points().unwrap() >= 1 {
        assert!(s.point_state(1).unwrap() >= 0);
    }
}
#[test]
fn math_FunctionAllRoots__LargeSampleSize() {
    let mut f = CubicFunction;
    let sample = FunctionSample::new(0.0, 5.0, 101);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-8, 1e-8, 1e-10]).unwrap();
    assert!(s.is_done());
    assert!(s.number_of_points().unwrap() >= 2);
}
#[test]
fn math_FunctionAllRoots__SmallSampleSize() {
    let mut f = CubicFunction;
    let sample = FunctionSample::new(0.0, 5.0, 5);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-4, 1e-4, 1e-6]).unwrap();
    assert!(s.is_done());
    assert!(s.number_of_points().unwrap() >= 0);
}
#[test]
fn math_FunctionAllRoots__TightTolerances() {
    let mut f = QuadraticFunction;
    let sample = FunctionSample::new(0.0, 3.0, 31);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-10, 1e-10, 1e-12]).unwrap();
    assert!(s.is_done());
    assert!(s.number_of_points().unwrap() >= 1);
}
#[test]
fn math_FunctionAllRoots__LooseTolerances() {
    let mut f = CubicFunction;
    let sample = FunctionSample::new(0.0, 5.0, 21);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-2, 1e-2, 1e-4]).unwrap();
    assert!(s.is_done());
    assert!(s.number_of_points().unwrap() >= 0);
}
#[test]
fn math_FunctionAllRoots__NarrowRange() {
    let mut f = QuadraticFunction;
    let sample = FunctionSample::new(0.5, 2.5, 21);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-6, 1e-6, 1e-8]).unwrap();
    assert!(s.is_done());
    assert!(s.number_of_points().unwrap() >= 1);
}
#[test]
fn math_FunctionAllRoots__WideRange() {
    let mut f = SineFunction;
    let sample = FunctionSample::new(-10.0, 10.0, 81);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-6, 1e-6, 1e-8]).unwrap();
    assert!(s.is_done());
    assert!(s.number_of_points().unwrap() >= 5);
}
#[test]
fn math_FunctionAllRoots__EmptyRange() {
    let mut f = CubicFunction;
    let sample = FunctionSample::new(10.0, 20.0, 11);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-6, 1e-6, 1e-8]).unwrap();
    assert!(s.is_done());
    assert_eq!(s.number_of_points().unwrap(), 0);
    assert_eq!(s.number_of_intervals().unwrap(), 0);
}
#[test]
fn math_FunctionAllRoots__IntervalBounds() {
    let mut f = NullIntervalFunction;
    let sample = FunctionSample::new(0.0, 6.0, 31);
    let s = FunctionAllRoots::new(&mut f, &sample, [1e-6, 1e-6, 1e-6]).unwrap();
    assert!(s.is_done());
    if s.number_of_intervals().unwrap() >= 1 {
        let [a, b] = s.interval(1).unwrap();
        assert!(a < b);
        assert!(a >= 0.0);
        assert!(b <= 6.0);
    }
}
#[test]
fn math_FunctionAllRoots__MultiplePrecisionLevels() {
    let mut f = CubicFunction;
    for tolerance in [1e-4, 1e-6, 1e-8] {
        let sample = FunctionSample::new(0.0, 5.0, 21);
        let s = FunctionAllRoots::new(&mut f, &sample, [tolerance, tolerance, tolerance * 0.01])
            .unwrap();
        assert!(s.is_done());
        assert!(s.number_of_points().unwrap() >= 0);
    }
}
