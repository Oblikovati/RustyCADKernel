//! Checks native evaluator input mutations across the caller's repeated samples.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkg3d::adv_approx::{
    AdvApproxEvaluation, AdvApproxEvaluatorFunction, AdvApproxSimpleApprox,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::plib::{self, JacobiPolynomial};

#[derive(Debug, PartialEq)]
struct CallRecord {
    dimension: i32,
    start_end: [f64; 2],
    parameter: f64,
    derivative: i32,
    result: Vec<f64>,
}

struct MutatingEvaluator {
    calls: Vec<CallRecord>,
}

impl AdvApproxEvaluatorFunction for MutatingEvaluator {
    fn evaluate(&mut self, _: i32, _: [f64; 2], _: f64, _: i32, _: &mut [f64]) -> i32 {
        unreachable!("the native mutable hook handles every callback")
    }

    fn evaluate_native(&mut self, call: &mut AdvApproxEvaluation<'_>) {
        self.mutate_first_endpoint(call);
        self.mutate_gauss_derivative(call);
        self.write_linear_values(call);
        let dimension = *call.dimension;
        self.record(call, dimension);
        self.mutate_dimension_after_full_sample(call);
    }
}

impl MutatingEvaluator {
    fn mutate_first_endpoint(&mut self, call: &mut AdvApproxEvaluation<'_>) {
        if self.calls.is_empty() {
            call.start_end[1] = 2.0;
            *call.parameter = 0.25;
        }
    }

    fn mutate_gauss_derivative(&self, call: &mut AdvApproxEvaluation<'_>) {
        if self.calls.len() == 4 {
            *call.derivative_request = 1;
        }
    }

    fn record(&mut self, call: &AdvApproxEvaluation<'_>, dimension: i32) {
        self.calls.push(CallRecord {
            dimension,
            start_end: *call.start_end,
            parameter: *call.parameter,
            derivative: *call.derivative_request,
            result: call.result.to_vec(),
        });
    }

    fn mutate_dimension_after_full_sample(&self, call: &mut AdvApproxEvaluation<'_>) {
        if self.calls.len() == 5 {
            *call.dimension = 1;
        }
    }

    fn write_linear_values(&self, call: &mut AdvApproxEvaluation<'_>) {
        for index in 0..*call.dimension as usize {
            call.result[index] = *call.parameter + index as f64;
        }
        *call.error_code = 0;
    }
}

fn table(values: &[f64]) -> Array1<'static, f64> {
    let mut result = Array1::with_bounds(1, values.len() as i32).unwrap();
    result.as_mut_slice().copy_from_slice(values);
    result
}

#[test]
fn mutable_native_inputs_persist_across_simple_approx_samples() {
    let (mut gauss, mut degree) = (0, 0);
    plib::jacobi_parameters(Shape::C1, 13, 1, &mut gauss, &mut degree).unwrap();
    let jacobi = JacobiPolynomial::new(degree, Shape::C1).unwrap();
    let mut evaluator = MutatingEvaluator { calls: Vec::new() };
    let mut approximation =
        AdvApproxSimpleApprox::new(2, 1, Shape::C1, degree, gauss, &jacobi, &mut evaluator)
            .unwrap();
    let mut dimensions = Array1::with_bounds(1, 1).unwrap();
    dimensions[1] = 2;
    approximation
        .perform(&dimensions, &table(&[1e-6]), 0.0, 1.0, 13)
        .unwrap();
    assert_mutations_reach_later_samples(&evaluator.calls);
}

fn assert_mutations_reach_later_samples(calls: &[CallRecord]) {
    assert_eq!(calls.len(), 34);
    assert_eq!(calls[0].parameter, 0.25);
    assert_eq!(calls[1].parameter, 0.25);
    assert_eq!(calls[1].start_end, [0.0, 2.0]);
    assert_eq!(calls[4].derivative, 1);
    assert_eq!(calls[5].derivative, 1);
    assert_eq!((calls[4].dimension, calls[5].dimension), (2, 1));
    assert_eq!(calls[4].result[1], calls[5].result[1]);
}
