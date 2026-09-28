//! Standard_Real limits, conversions, Epsilon, ACosApprox and the deprecated `<cmath>` wrappers.
use rustycad_tkernel::standard::real;

const MIN_SUBNORMAL: f64 = 5e-324;

#[test]
fn limits_match_float_h() {
    assert_eq!(real::real_small(), 2.2250738585072014e-308);
    assert_eq!(real::real_digits(), 15);
    assert_eq!(real::real_epsilon(), 2.220446049250313e-16);
    assert_eq!(real::real_first(), -1.7976931348623157e308);
    assert_eq!(real::real_first10_exp(), -307);
    assert_eq!(real::real_last(), 1.7976931348623157e308);
    assert_eq!(real::real_last10_exp(), 308);
    assert_eq!(real::real_mantissa(), 53);
    assert_eq!(real::real_radix(), 2);
    assert_eq!(real::real_size(), 64);
}

#[test]
fn epsilon_steps_away_from_zero_and_keeps_nan_sign() {
    assert_eq!(real::epsilon(0.0), MIN_SUBNORMAL);
    assert_eq!(real::epsilon(-0.0), MIN_SUBNORMAL);
    assert_eq!(real::epsilon(-1.0), f64::EPSILON);
    assert_eq!(real::epsilon(f64::MAX), 0.0);
    assert_eq!(real::epsilon(f64::INFINITY), f64::NEG_INFINITY);
    assert_eq!(real::epsilon(f64::NEG_INFINITY), f64::NEG_INFINITY);
    // A negative NaN stays negative (the former `abs()` formulation cleared its sign bit).
    let negative_nan = f64::from_bits(0xfff8_0000_0000_0000);
    assert_eq!(
        real::epsilon(negative_nan).to_bits(),
        negative_nan.to_bits()
    );
}

#[test]
fn next_after_follows_c_nextafter() {
    assert_eq!(real::next_after(0.0, 1.0), MIN_SUBNORMAL);
    assert_eq!(real::next_after(0.0, -1.0), -MIN_SUBNORMAL);
    assert_eq!(real::next_after(0.0, -0.0).to_bits(), (-0.0_f64).to_bits());
    assert_eq!(real::next_after(f64::INFINITY, 0.0), f64::MAX);
    assert_eq!(real::next_after(f64::MAX, f64::INFINITY), f64::INFINITY);
    let payload = f64::from_bits(0x7ff8_0000_0000_0123);
    let negative_nan = f64::from_bits(0xfff8_0000_0000_0000);
    // Both NaN: glibc keeps the direction's payload.
    assert_eq!(
        real::next_after(payload, negative_nan).to_bits(),
        negative_nan.to_bits()
    );
    assert_eq!(real::next_after(payload, 1.0).to_bits(), payload.to_bits());
}

#[test]
fn real_to_int_clamps_and_truncates() {
    assert_eq!(real::real_to_int(2147483647.5), i32::MAX);
    assert_eq!(real::real_to_int(2147483648.0), i32::MAX);
    assert_eq!(real::real_to_int(-2147483648.5), i32::MIN);
    assert_eq!(real::real_to_int(-2.9), -2);
    assert_eq!(real::real_to_int(f64::INFINITY), i32::MAX);
    // NaN is undefined behavior natively; the port returns the x86-64 cvttsd2si value.
    assert_eq!(real::real_to_int(f64::NAN), i32::MIN);
}

#[test]
fn real_to_short_real_clamps_only_beyond_flt_max() {
    assert_eq!(real::real_to_short_real(1e300), f32::MAX);
    assert_eq!(real::real_to_short_real(-1e300), -f32::MAX);
    assert_eq!(real::real_to_short_real(f64::NEG_INFINITY), -f32::MAX);
    // Just above FLT_MAX but still rounding to it: clamped to FLT_MAX, never infinity.
    assert_eq!(real::real_to_short_real(3.4028235677973362e38), f32::MAX);
    assert_eq!(real::real_to_short_real(0.1), 0.1_f32);
    assert!(real::real_to_short_real(f64::NAN).is_nan());
}

#[test]
fn conversions_and_simple_arithmetic() {
    assert_eq!(real::int_to_real(i32::MIN), -2147483648.0);
    assert_eq!(real::real_part(-2.25), 0.25);
    assert!(real::real_part(f64::INFINITY).is_nan());
    assert_eq!(real::square(-1.5), 2.25);
    assert!(real::is_equal(1.0, 1.0 + 1e-320));
    assert!(!real::is_equal(1.0, 1.0 + f64::EPSILON));
    assert!(!real::is_equal(f64::NAN, f64::NAN));
}

#[test]
fn acos_approx_matches_the_polynomial_and_its_end_points() {
    assert_eq!(real::acos_approx(1.0), 0.0);
    // At -1 the native code returns 0, not PI.
    assert_eq!(real::acos_approx(-1.0), 0.0);
    // Native bits; the header's "about 1 degree" is 1.23 degrees at 0.
    assert_eq!(real::acos_approx(0.0).to_bits(), 0x3ff8_ca7d_bf2f_5385);
    assert_eq!(real::acos_approx(0.5).to_bits(), 0x3ff0_bd77_5b40_5d95);
    assert_eq!(real::acos_approx(-0.5).to_bits(), 0x4000_c33f_a6a3_fe4e);
    assert!(real::acos_approx(f64::NAN).is_nan());
}

#[test]
fn max_and_min_keep_std_comparison_semantics() {
    assert!(real::max(f64::NAN, 1.0).is_nan());
    assert_eq!(real::max(1.0, f64::NAN), 1.0);
    assert!(real::min(f64::NAN, 1.0).is_nan());
    assert_eq!(real::min(1.0, f64::NAN), 1.0);
    assert_eq!(real::max(-0.0, 0.0).to_bits(), (-0.0_f64).to_bits());
    assert_eq!(real::min(0.0, -0.0).to_bits(), 0.0_f64.to_bits());
}

#[test]
fn elementary_wrappers_call_the_c_library() {
    assert_eq!(real::acos(-1.0), std::f64::consts::PI);
    assert_eq!(real::asin(1.0), std::f64::consts::FRAC_PI_2);
    assert_eq!(real::atan(f64::INFINITY), std::f64::consts::FRAC_PI_2);
    assert_eq!(real::atan2(1.0, 0.0), std::f64::consts::FRAC_PI_2);
    assert_eq!(real::atanh(1.0), f64::INFINITY);
    assert!(real::atanh(2.0).is_nan());
    assert!(real::acosh(0.5).is_nan());
    assert_eq!(real::asinh(-0.0).to_bits(), (-0.0_f64).to_bits());
    assert_eq!(real::cosh(0.0), 1.0);
    assert_eq!(real::sinh(711.0), f64::INFINITY);
    assert_eq!(real::tanh(f64::NEG_INFINITY), -1.0);
    assert_eq!(
        (real::cos(0.0), real::sin(0.0), real::tan(0.0)),
        (1.0, 0.0, 0.0)
    );
    assert_eq!(
        (real::exp(0.0), real::log(1.0), real::log10(1000.0)),
        (1.0, 0.0, 3.0)
    );
    assert!(real::pow(-8.0, 1.0 / 3.0).is_nan());
    assert_eq!(real::sqrt(-0.0).to_bits(), (-0.0_f64).to_bits());
    assert_eq!(real::sign(3.0, -0.0), -3.0);
    assert_eq!(real::abs(-0.0).to_bits(), 0);
    assert_eq!((real::ceiling(-0.5), real::floor(-0.5)), (-0.0, -1.0));
    assert_eq!((real::integer_part(2.7), real::round(0.5)), (2.0, 1.0));
}
