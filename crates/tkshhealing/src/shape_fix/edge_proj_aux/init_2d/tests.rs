use super::{distance_is_within, native_max, native_min};

#[test]
fn native_bounds_preserve_left_operand_when_either_operand_is_nan() {
    let nan = f64::NAN;
    assert!(native_max(nan, -23.0).is_nan());
    assert!(native_max(23.0, nan).is_finite());
    assert!(native_min(nan, 23.0).is_nan());
    assert!(native_min(-23.0, nan).is_finite());
}

#[test]
fn native_is_equal_rejects_nan_distance_and_tolerance() {
    assert!(distance_is_within(0.0, 1.0));
    assert!(!distance_is_within(2.0, 1.0));
    assert!(!distance_is_within(f64::NAN, 1.0));
    assert!(!distance_is_within(0.0, f64::NAN));
}
