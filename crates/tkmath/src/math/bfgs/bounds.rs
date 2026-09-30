use super::Vector;
const PCONFUSION: f64 = 1e-9;
pub(super) fn native_min(first: f64, second: f64) -> f64 {
    if second < first { second } else { first }
}
pub(super) fn native_max(first: f64, second: f64) -> f64 {
    if first < second { second } else { first }
}
pub(super) fn initial_scale(
    value: f64,
    direction: &Vector<'_>,
    gradient: &Vector<'_>,
) -> Option<f64> {
    let slope: f64 = gradient.dot_unchecked(direction);
    if slope.abs() < f64::MIN_POSITIVE {
        return None;
    }
    let norm: f64 = direction.norm2_unchecked();
    let alpha: f64 = 0.7 * (-value) / slope;
    let scale: f64 = 0.015 / norm.sqrt();
    Some(if scale > alpha { alpha } else { scale })
}
pub(super) fn limit_scales(
    point: &Vector<'_>,
    direction: &Vector<'_>,
    left: &Vector<'_>,
    right: &Vector<'_>,
    limits: &mut [f64; 2],
) -> bool {
    for index in 1..=left.upper() {
        if !limit_coordinate(
            left[index] - point[index],
            right[index] - point[index],
            direction[index],
            limits,
        ) {
            return false;
        }
    }
    true
}
fn limit_coordinate(left: f64, right: f64, direction: f64, limits: &mut [f64; 2]) -> bool {
    if direction.abs() <= f64::MIN_POSITIVE || direction.is_nan() {
        return !(left > PCONFUSION || right < -PCONFUSION);
    }
    let low: f64 = (left + PCONFUSION) / direction;
    let high: f64 = (right - PCONFUSION) / direction;
    let pair: (f64, f64) = if left.abs() < PCONFUSION {
        (0.0, high)
    } else if right.abs() < PCONFUSION {
        (0.0, low)
    } else if left * right < 0.0 {
        (low, high)
    } else {
        return false;
    };
    limits[1] = native_min(limits[1], native_max(pair.0, pair.1));
    limits[0] = native_max(limits[0], native_min(pair.0, pair.1));
    true
}
pub(super) fn tangent_direction(
    point: &Vector<'_>,
    direction: &mut Vector<'_>,
    left: &Vector<'_>,
    right: &Vector<'_>,
) {
    for index in 1..=left.upper() {
        if ((point[index] - right[index]).abs() < PCONFUSION && direction[index] > 0.0)
            || ((point[index] - left[index]).abs() < PCONFUSION && direction[index] < 0.0)
        {
            direction[index] = 0.0;
        }
    }
}
