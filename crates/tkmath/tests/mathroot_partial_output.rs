//! OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 golden search retains callback output.
use rustycad_tkmath::{
    math::{Function, FunctionWithDerivative},
    math_root::*,
};

#[derive(Default)]
struct PartialOutputPolynomial {
    calls: Vec<(i32, u64)>,
}
impl PartialOutputPolynomial {
    fn record(&mut self, kind: i32, x: f64, value: &mut f64) -> bool {
        self.calls.push((kind, x.to_bits()));
        if !self.calls.len().is_multiple_of(3) {
            *value = x * x - 0.37165122816093876;
        }
        true
    }
}
impl Function for PartialOutputPolynomial {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        self.record(0, x, value)
    }
}
impl FunctionWithDerivative for PartialOutputPolynomial {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = 2.0 * x;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *derivative = 2.0 * x;
        self.record(1, x, value)
    }
}
fn native_trace() -> Vec<(i32, u64)> {
    include_str!("fixtures/multiple_partial_output_trace.txt")
        .lines()
        .map(|line| {
            let (kind, bits): (&str, &str) = line.split_once(' ').unwrap();
            (kind.parse().unwrap(), bits.parse().unwrap())
        })
        .collect()
}
#[test]
fn golden_search_retains_outputs_left_untouched_by_callback() {
    let mut objective: PartialOutputPolynomial = PartialOutputPolynomial::default();
    let config: MultipleConfig = MultipleConfig {
        nb_samples: 30,
        x_tolerance: 1e-8,
        f_tolerance: 1e-12,
        null_tolerance: 1e-12,
        max_iterations: 1,
        offset: -0.5,
    };
    let result: MultipleResult = find_all_roots_with_derivative(&mut objective, 0.0, 7.0, &config);
    assert!(result.is_done());
    assert_eq!(result.iterations, 0);
    assert!(!result.is_all_null);
    assert!(result.roots.is_empty());
    assert_eq!(objective.calls, native_trace());
}
#[test]
fn polynomial_derivative_matches_native_formula() {
    let mut objective: PartialOutputPolynomial = PartialOutputPolynomial::default();
    let mut derivative: f64 = 0.0;
    assert!(objective.derivative(2.0, &mut derivative));
    assert_eq!(derivative, 4.0);
}
