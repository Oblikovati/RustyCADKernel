//! Period shifts use ShapeAnalysis's floor, unlike ShapeBuild_Edge's integer cast.

/// Shift to the closest equivalent parameter; `adjust_by_period(7.0, 0.0, 6.0) == -6.0`.
pub fn adjust_by_period(value: f64, target: f64, period: f64) -> f64 {
    let difference = value - target;
    let distance = difference.abs();
    let period = period.abs();
    if distance <= 0.5 * period {
        return 0.0;
    }
    if period < 1e-100 {
        return difference;
    }
    (if difference > 0.0 { -period } else { period }) * (distance / period + 0.5).floor()
}
/// Shift toward the middle of one period; `adjust_to_period(7.0, 0.0, 6.0) == -6.0`.
pub fn adjust_to_period(value: f64, first: f64, last: f64) -> f64 {
    adjust_by_period(value, 0.5 * (first + last), last - first)
}
