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

/// `static_cast<int>(float)` as compiled for x86-64 (`cvttss2si`), with the same
/// `INT_MIN` result for NaN and out-of-range values; `int_from_f32(254.9) == 254`.
pub(crate) fn int_from_f32(value: f32) -> i32 {
    int_from_f64(f64::from(value))
}

/// `(unsigned int)double` as compiled for x86-64: a 64-bit `cvttsd2si` whose low 32 bits
/// are kept, so a negative value wraps and NaN or a value outside the `long` range gives 0.
///
/// Example: `uint_from_f64(2.7) == 2`, `uint_from_f64(-1.5) == u32::MAX`.
pub(crate) fn uint_from_f64(value: f64) -> u32 {
    let truncated = value.trunc();
    let long_range = 9_223_372_036_854_775_808.0; // tol:exact 2^63, the `long` limit
    if truncated.is_nan() || truncated < -long_range || truncated >= long_range {
        return 0;
    }
    (truncated as i64) as u32
}

#[cfg(test)]
mod tests {
    use super::{int_from_f64, uint_from_f64};

    #[test]
    fn unsigned_conversion_keeps_the_low_32_bits() {
        assert_eq!(uint_from_f64(2.7), 2);
        assert_eq!(uint_from_f64(-1.5), u32::MAX);
        assert_eq!(uint_from_f64(-0.5), 0);
        assert_eq!(uint_from_f64(4294967296.0 + 7.0), 7);
        assert_eq!(uint_from_f64(f64::NAN), 0);
        assert_eq!(uint_from_f64(1e300), 0);
    }

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
