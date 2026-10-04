use crate::adv_approx::evaluator::{AdvApproxEvaluation, AdvApproxEvaluatorFunction};

pub(super) struct EvaluationState {
    pub(super) dimension: i32,
    pub(super) start_end: [f64; 2],
    pub(super) interval: [f64; 2],
    pub(super) fact: f64,
    error_code: i32,
}

impl EvaluationState {
    pub(super) fn new(dimension: i32, start_end: [f64; 2]) -> Self {
        Self {
            dimension,
            start_end,
            interval: start_end,
            fact: (start_end[1] - start_end[0]) / 2.0,
            error_code: 0,
        }
    }

    pub(super) fn evaluate(
        &mut self,
        evaluator: &mut dyn AdvApproxEvaluatorFunction,
        parameter: &mut f64,
        derivative: &mut i32,
        result: &mut [f64],
    ) -> bool {
        let mut call = self.call(parameter, derivative, result);
        evaluator.evaluate_native(&mut call);
        self.error_code == 0
    }

    fn call<'a>(
        &'a mut self,
        parameter: &'a mut f64,
        derivative: &'a mut i32,
        result: &'a mut [f64],
    ) -> AdvApproxEvaluation<'a> {
        AdvApproxEvaluation {
            dimension: &mut self.dimension,
            start_end: &mut self.start_end,
            parameter,
            derivative_request: derivative,
            result,
            error_code: &mut self.error_code,
        }
    }
}
