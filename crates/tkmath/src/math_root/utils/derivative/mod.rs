//! `FindAllRootsWithDerivativeImpl` of MathRoot_MultipleUtils.hxx, split into its stages
//! (sign changes, exact zero samples, sampled extrema) with the native callback order.
use super::{
    add_root, effective_x_tolerance, evaluate_shifted_value, evaluate_shifted_values,
    evaluate_value, is_valid_multiple_input, multiple_sample_count, refine_bracketed_root,
    sort_roots,
};
use crate::math::FunctionWithDerivative;
use crate::math_root::{MultipleConfig, MultipleResult};
use crate::math_utils::Status;
mod extrema;
mod stages;

/// Derivative-aware multi-root search mirroring math_FunctionRoots (`FindAllRootsWithDerivativeImpl`):
/// Brent+Newton on sign changes, bracketing of exact zero samples, endpoint residual checks and
/// derivative/golden-section searches at sampled extrema that nearly touch zero.
/// `find_all_roots_with_derivative_impl(&mut f, 0.0, 1.0, &MultipleConfig::default())`.
pub fn find_all_roots_with_derivative_impl<F: FunctionWithDerivative>(
    function: &mut F,
    lower: f64,
    upper: f64,
    config: &MultipleConfig,
) -> MultipleResult {
    if !is_valid_multiple_input(lower, upper, config) {
        return MultipleResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let mut search: Search<'_, F> = Search::new(function, lower, upper, config);
    if let Err(status) = search.derivative_search() {
        search.result.status = status;
    }
    search.result
}

/// Working state shared by the stages; `result` keeps roots found before a failure.
struct Search<'a, F> {
    function: &'a mut F,
    config: MultipleConfig,
    lower: f64,
    upper: f64,
    count: usize,
    epsilon: f64,
    dx: f64,
    xs: Vec<f64>,
    ys: Vec<f64>,
    result: MultipleResult,
}

fn checked(status: Status) -> Result<(), Status> {
    if status == Status::Ok {
        Ok(())
    } else {
        Err(status)
    }
}

impl<'a, F: FunctionWithDerivative> Search<'a, F> {
    fn new(function: &'a mut F, lower: f64, upper: f64, config: &MultipleConfig) -> Self {
        let count: usize = multiple_sample_count(config.nb_samples);
        Self {
            function,
            config: *config,
            lower,
            upper,
            count,
            epsilon: effective_x_tolerance(lower, upper, config.x_tolerance),
            dx: upper / count as f64 - lower / count as f64,
            xs: Vec::with_capacity(count + 1),
            ys: Vec::with_capacity(count + 1),
            result: MultipleResult {
                status: Status::Ok,
                ..Default::default()
            },
        }
    }
    fn value(&mut self, x: f64) -> Result<f64, Status> {
        let mut value: f64 = 0.0;
        checked(evaluate_value(self.function, x, &mut value))?;
        Ok(value)
    }
    /// Writes into `value` even on failure, as the C++ output reference does.
    fn shifted_into(&mut self, x: f64, value: &mut f64) -> Result<(), Status> {
        checked(evaluate_shifted_value(
            self.function,
            x,
            self.config.offset,
            value,
        ))
    }
    fn shifted(&mut self, x: f64) -> Result<f64, Status> {
        let mut value: f64 = 0.0;
        self.shifted_into(x, &mut value)?;
        Ok(value)
    }
    fn shifted_values_into(
        &mut self,
        x: f64,
        value: &mut f64,
        derivative: &mut f64,
    ) -> Result<(), Status> {
        let offset: f64 = self.config.offset;
        checked(evaluate_shifted_values(
            self.function,
            x,
            offset,
            value,
            derivative,
        ))
    }
    fn add_at(&mut self, x: f64) -> Result<(), Status> {
        let value: f64 = self.value(x)?;
        add_root(&mut self.result, self.epsilon, x, value);
        Ok(())
    }
    fn refined_bracket(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> Result<(), Status> {
        checked(refine_bracketed_root(
            self.function,
            self.config.offset,
            x1,
            y1,
            x2,
            y2,
            self.epsilon,
            self.config.f_tolerance,
            self.config.max_iterations,
            self.epsilon,
            &mut self.result,
        ))
    }
    fn derivative_search(&mut self) -> Result<(), Status> {
        self.sample()?;
        if self.result.is_all_null {
            return Ok(());
        }
        self.crossing_samples()?;
        self.sampled_zeros()?;
        self.extrema_search()?;
        sort_roots(&mut self.result);
        Ok(())
    }
}
