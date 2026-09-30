//! MathInteg_Gauss.hxx adaptive rule at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::gauss::{gauss, valid_interval};
use crate::{
    math::Function,
    math_utils::{IntegConfig, IntegResult, Status},
};
#[derive(Clone, Copy, Default)]
struct Interval {
    lower: f64,
    upper: f64,
    value: f64,
    error: f64,
}
struct AdaptiveSum {
    intervals: Vec<Interval>,
    value: f64,
    error: f64,
    points: usize,
    work: u32,
}
/// Refines the largest estimated error first; `gauss_adaptive(&mut f, 0.0, 1.0, &IntegConfig::default())`.
pub fn gauss_adaptive(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    config: &IntegConfig,
) -> IntegResult {
    if !valid_interval(lower, upper) || !valid_config(config) {
        return IntegResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    if lower == upper {
        return IntegResult {
            status: Status::Ok,
            value: Some(0.0),
            absolute_error: Some(0.0),
            ..Default::default()
        };
    }
    if lower > upper {
        let mut result: IntegResult = gauss_adaptive(f, upper, lower, config);
        result.value = result.value.map(|value| -value);
        return result;
    }
    adaptive_work(f, lower, upper, config)
}
fn valid_config(config: &IntegConfig) -> bool {
    config.tolerance.is_finite()
        && config.tolerance > 0.0
        && config.initial_order >= 1
        && config.max_order >= config.initial_order
        && config.max_order >= 2
        && config.max_order as usize <= super::gauss::GAUSS_MAX_ORDER
        && config.max_iterations >= 1
}
fn paired_orders(config: &IntegConfig) -> [usize; 2] {
    let mut coarse: u32 = config.initial_order;
    let fine: u32 = config
        .max_order
        .min((super::gauss::GAUSS_MAX_ORDER as u32).min(2 * coarse));
    if fine == coarse {
        coarse -= 1;
    }
    [coarse as usize, fine as usize]
}
fn evaluate_interval(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    orders: [usize; 2],
    interval: &mut Interval,
) -> IntegResult {
    let coarse: IntegResult = gauss(f, lower, upper, orders[0]);
    if !coarse.is_done() {
        return coarse;
    }
    let mut fine: IntegResult = gauss(f, lower, upper, orders[1]);
    fine.number_of_points += coarse.number_of_points;
    if fine.is_done() {
        *interval = Interval {
            lower,
            upper,
            value: fine.value.unwrap(),
            error: (fine.value.unwrap() - coarse.value.unwrap()).abs(),
        };
    }
    fine
}
pub(super) fn converged(error: f64, value: f64, tolerance: f64) -> bool {
    if value.abs() > f64::EPSILON {
        error <= tolerance * value.abs()
    } else {
        error <= tolerance
    }
}
fn adaptive_work(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    config: &IntegConfig,
) -> IntegResult {
    integrate_adaptive(f, lower, upper, config, Rule::Gauss(paired_orders(config)))
}
pub(super) fn integrate_adaptive(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    config: &IntegConfig,
    rule: Rule,
) -> IntegResult {
    let mut initial: Interval = Interval::default();
    let result: IntegResult = rule.evaluate(f, lower, upper, &mut initial);
    if !result.is_done() {
        return result;
    }
    AdaptiveSum::new(initial, result.number_of_points).refine(f, rule, config)
}
impl AdaptiveSum {
    fn new(initial: Interval, points: usize) -> Self {
        Self {
            intervals: vec![initial],
            value: initial.value,
            error: initial.error,
            points,
            work: 1,
        }
    }
    fn refine(mut self, f: &mut impl Function, rule: Rule, config: &IntegConfig) -> IntegResult {
        let mut status: Status = Status::NotConverged;
        while !converged(self.error, self.value, config.tolerance) {
            if self.work >= config.max_iterations {
                status = Status::MaxIterations;
                break;
            }
            if let Err(failure) = self.subdivide(f, rule) {
                status = failure;
                break;
            }
        }
        if status == Status::NotConverged && converged(self.error, self.value, config.tolerance) {
            status = Status::Ok;
        }
        self.result(status)
    }
}
impl AdaptiveSum {
    fn worst_index(&self) -> usize {
        let mut worst: usize = 0;
        for i in 1..self.intervals.len() {
            if self.intervals[i].error > self.intervals[worst].error {
                worst = i;
            }
        }
        worst
    }
    fn subdivide(&mut self, f: &mut impl Function, rule: Rule) -> Result<(), Status> {
        let index: usize = self.worst_index();
        let worst: Interval = self.intervals[index];
        let mid: f64 = worst.lower + 0.5 * (worst.upper - worst.lower);
        if !(mid > worst.lower && mid < worst.upper) {
            return Err(Status::NotConverged);
        }
        let left: Interval = self.child(f, worst.lower, mid, rule)?;
        let right: Interval = self.child(f, mid, worst.upper, rule)?;
        self.update_totals(worst, left, right, rule);
        self.intervals[index] = left;
        self.intervals.push(right);
        self.work += 1;
        Ok(())
    }
    fn child(
        &mut self,
        f: &mut impl Function,
        lower: f64,
        upper: f64,
        rule: Rule,
    ) -> Result<Interval, Status> {
        let mut interval: Interval = Interval::default();
        let result: IntegResult = rule.evaluate(f, lower, upper, &mut interval);
        self.points += result.number_of_points;
        if !result.is_done() {
            return Err(result.status);
        }
        Ok(interval)
    }
    fn result(self, status: Status) -> IntegResult {
        IntegResult {
            status,
            value: Some(self.value),
            absolute_error: Some(self.error),
            relative_error: (self.value.abs() > f64::EPSILON)
                .then_some(self.error / self.value.abs()),
            number_of_points: self.points,
            iterations: self.work,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Rule {
    Gauss([usize; 2]),
    Kronrod(usize),
}
impl Rule {
    fn evaluate(
        self,
        f: &mut impl Function,
        lower: f64,
        upper: f64,
        interval: &mut Interval,
    ) -> IntegResult {
        match self {
            Self::Gauss(orders) => evaluate_interval(f, lower, upper, orders, interval),
            Self::Kronrod(order) => {
                let result: IntegResult = super::kronrod_rule(f, lower, upper, order);
                if result.is_done() {
                    *interval = Interval {
                        lower,
                        upper,
                        value: result.value.unwrap(),
                        error: result.absolute_error.unwrap(),
                    };
                }
                result
            }
        }
    }
}
impl AdaptiveSum {
    fn update_totals(&mut self, worst: Interval, left: Interval, right: Interval, rule: Rule) {
        if matches!(rule, Rule::Kronrod(_)) {
            // OCCT's Kronrod totals subtract before adding; reassociation changes finite bits.
            self.value -= worst.value;
            self.error -= worst.error;
            self.value += left.value + right.value;
            self.error += left.error + right.error;
            self.error = self.error.max(0.0);
            return;
        }
        self.value += left.value + right.value - worst.value;
        self.error = (self.error + left.error + right.error - worst.error).max(0.0);
    }
}
