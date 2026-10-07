//! Native numeric conversions of the oracle platform.

/// `static_cast<int>(double)` as compiled for x86-64 (`cvttsd2si`): truncation toward zero,
/// and `INT_MIN` for NaN and for values outside the `int` range, where Rust's `as`
/// saturates (and gives 0 for NaN) instead.
///
/// Example: `int_from_f64(2.7) == 2`, `int_from_f64(f64::NAN) == i32::MIN`.
pub(crate) fn int_from_f64(value: f64) -> i32 {
    let truncated = value.trunc();
    if truncated.is_nan() || truncated < f64::from(i32::MIN) || truncated > f64::from(i32::MAX) {
        return i32::MIN;
    }
    truncated as i32
}

#[cfg(test)]
mod tests {
    use super::int_from_f64;

    #[test]
    fn truncates_and_reports_out_of_range_as_int_min() {
        assert_eq!(int_from_f64(2.7), 2);
        assert_eq!(int_from_f64(-2.7), -2);
        assert_eq!(int_from_f64(f64::NAN), i32::MIN);
        assert_eq!(int_from_f64(3.0e10), i32::MIN);
        assert_eq!(int_from_f64(-3.0e10), i32::MIN);
        assert_eq!(int_from_f64(2147483647.9), i32::MAX);
    }
}
