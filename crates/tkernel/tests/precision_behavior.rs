//! Precision has no upstream GoogleTest suite; these pin every function to the
//! bits of its native constexpr expression and the infinity boundaries.
use rustycad_tkernel::precision::*;

#[test]
fn constants_keep_native_expression_bits() {
    let confusion_bits: u64 = 1.0e-7_f64.to_bits();
    assert_eq!(confusion().to_bits(), confusion_bits);
    assert_eq!(angular().to_bits(), 1.0e-12_f64.to_bits());
    assert_eq!(
        square_confusion().to_bits(),
        (1.0e-7_f64 * 1.0e-7).to_bits()
    );
    assert_eq!(computational(), f64::EPSILON);
    assert_eq!(square_computational(), f64::EPSILON * f64::EPSILON);
    assert_eq!(intersection().to_bits(), (1.0e-7_f64 * 0.01).to_bits());
    assert_eq!(approximation().to_bits(), (1.0e-7_f64 * 10.0).to_bits());
    assert_eq!(infinite(), 2.0e100);
}

#[test]
fn parametric_forms_divide_or_scale_like_native() {
    assert_eq!(parametric(3.0), 3.0 * 0.01);
    assert_eq!(parametric_with_speed(3.0, 7.0), 3.0 / 7.0);
    assert_eq!(p_confusion(), confusion() * 0.01);
    assert_eq!(p_confusion_with_speed(7.0), confusion() / 7.0);
    assert_eq!(p_intersection(), intersection() * 0.01);
    assert_eq!(p_intersection_with_speed(7.0), intersection() / 7.0);
    assert_eq!(p_approximation(), approximation() * 0.01);
    assert_eq!(p_approximation_with_speed(7.0), approximation() / 7.0);
    assert_eq!(square_p_confusion(), p_confusion() * p_confusion());
    // Division by a zero speed follows IEEE rules, as the native constexpr does.
    assert_eq!(p_confusion_with_speed(0.0), f64::INFINITY);
}

#[test]
fn infinity_tests_use_half_of_infinite_as_boundary() {
    assert!(is_infinite(1.0e100) && is_infinite(-1.0e100) && is_infinite(f64::INFINITY));
    assert!(!is_infinite(9.999_999_999_999_999e99) && !is_infinite(f64::NAN));
    assert!(is_positive_infinite(1.0e100) && !is_positive_infinite(-1.0e100));
    assert!(is_negative_infinite(-1.0e100) && !is_negative_infinite(1.0e100));
    assert!(!is_positive_infinite(f64::NAN) && !is_negative_infinite(f64::NAN));
}
