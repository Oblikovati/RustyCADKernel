// SPDX-License-Identifier: GPL-2.0-only
//! Closed-form determinants and exact powers of two avoid a rounded reference determinant.
use super::{frames::*, test_support::identity};
use std::cmp::Ordering;

#[test]
fn exact_matrix_admission_survives_cancellation_of_rounded_products() {
    let n = 4_503_599_627_370_496.;
    let mut frame = identity();
    frame.linear = [[n, n - 1., 0.], [n + 1., n, 0.], [0., 0., 1.]];
    assert_eq!(n * n - (n - 1.) * (n + 1.), 0.);
    let admitted = admit_tolerance_frame(frame).ok().unwrap();
    assert_eq!(admitted.unscaled_determinant_sign, Ordering::Greater);
    assert_eq!(admitted.placement.linear, frame.linear);
}

#[test]
fn separate_extreme_scales_do_not_underflow_or_overflow_matrix_admission() {
    for scale in [f64::from_bits(1), -f64::from_bits(1), f64::MAX, -f64::MAX] {
        let mut frame = identity();
        frame.scale = scale;
        let coefficient = if scale.abs() < 1. { 0.5 } else { 2. };
        frame.linear = [
            [coefficient, 0., 0.],
            [0., coefficient, 0.],
            [0., 0., coefficient],
        ];
        assert!(scale * coefficient == 0. || !(scale * coefficient).is_finite());
        let admitted = admit_tolerance_frame(frame).ok().unwrap();
        assert_eq!(admitted.placement.scale.to_bits(), scale.to_bits());
        assert_eq!(admitted.unscaled_determinant_sign, Ordering::Greater);
    }
}

#[test]
fn singular_matrix_and_zero_or_nonfinite_scale_are_typed_refusals() {
    let mut frame = identity();
    frame.linear[2] = frame.linear[1];
    assert!(
        matches!(admit_tolerance_frame(frame), Err(ToleranceFrameFailure::Singular { actual }) if actual == frame.linear)
    );
    for scale in [0., -0., f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        let mut frame = identity();
        frame.scale = scale;
        let Err(ToleranceFrameFailure::Scale { actual }) = admit_tolerance_frame(frame) else {
            panic!("expected scale cause")
        };
        assert_eq!(actual.to_bits(), scale.to_bits());
    }
}

#[test]
fn invalid_origin_and_matrix_preserve_coordinate_and_nan_payload() {
    let invalid = f64::from_bits(0x7ff8_0000_0000_1234);
    for axis in 0..3 {
        let mut frame = identity();
        frame.origin[axis] = invalid;
        let Err(ToleranceFrameFailure::Origin {
            axis: actual_axis,
            actual,
        }) = admit_tolerance_frame(frame)
        else {
            panic!("expected origin cause")
        };
        assert_eq!((actual_axis, actual.to_bits()), (axis, invalid.to_bits()));
    }
    for position in 0..9 {
        assert_invalid_matrix(position, invalid);
    }
}

fn assert_invalid_matrix(position: usize, invalid: f64) {
    let mut frame = identity();
    frame.linear[position / 3][position % 3] = invalid;
    let Err(ToleranceFrameFailure::Matrix(cause)) = admit_tolerance_frame(frame) else {
        panic!("expected original matrix cause")
    };
    assert_eq!(
        (cause.point, cause.axis, cause.value.to_bits()),
        (position / 3, position % 3, invalid.to_bits())
    );
    assert!(
        cause
            .to_string()
            .contains("expected a finite binary64 coordinate")
    );
}

#[test]
fn frame_failure_messages_include_actual_operand_and_expected_shape() {
    let failures = [
        ToleranceFrameFailure::Origin {
            axis: 2,
            actual: f64::INFINITY,
        },
        ToleranceFrameFailure::Scale { actual: 0. },
        ToleranceFrameFailure::Singular {
            actual: [[0.; 3]; 3],
        },
    ];
    for failure in failures {
        assert!(failure.to_string().contains("expected"));
    }
}
