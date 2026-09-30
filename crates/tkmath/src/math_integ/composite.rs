//! MathInteg_Gauss.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::gauss::{GAUSS_MAX_ORDER, gauss, valid_interval};
use crate::{
    math::Function,
    math_utils::{IntegResult, Status},
};
/// Fixed subinterval integration preserving failure estimates; `gauss_composite(&mut f, 0.0, 1.0, 10, 7)`.
pub fn gauss_composite(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    intervals: u32,
    order: usize,
) -> IntegResult {
    if intervals == 0 || order == 0 || !valid_interval(lower, upper) {
        return IntegResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    if lower == upper {
        return IntegResult {
            status: Status::Ok,
            value: Some(0.0),
            ..Default::default()
        };
    }
    if lower > upper {
        let mut result: IntegResult = gauss_composite(f, upper, lower, intervals, order);
        result.value = result.value.map(|value| -value);
        return result;
    }
    composite_intervals(f, lower, upper, intervals, order.min(GAUSS_MAX_ORDER))
}
fn composite_intervals(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    intervals: u32,
    order: usize,
) -> IntegResult {
    let width: f64 = (upper - lower) / intervals as f64;
    let mut result: IntegResult = IntegResult {
        value: Some(0.0),
        status: Status::Ok,
        ..Default::default()
    };
    for i in 0..intervals {
        let a: f64 = lower + i as f64 * width;
        let child: IntegResult = gauss(f, a, a + width, order);
        result.number_of_points += child.number_of_points;
        result.iterations = i + 1;
        if !child.is_done() {
            result.status = child.status;
            return result;
        }
        *result.value.as_mut().unwrap() += child.value.unwrap();
    }
    result
}
