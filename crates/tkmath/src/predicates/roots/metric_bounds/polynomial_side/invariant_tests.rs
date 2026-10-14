// SPDX-License-Identifier: GPL-2.0-only
use super::PolynomialSideFailure::WorkLimit;
use super::tests::*;
use super::*;

#[test]
fn reversal_and_translation_retain_the_unsigned_integral() {
    let poles = [[0., 0., 0.], [-1., 0., 0.], [1., 0., 0.]];
    let mut reverse = poles;
    reverse.reverse();
    let mut moved = identity();
    moved.origin = [f64::MAX, -f64::MAX, 3.];
    let translated = certify_polynomial_side_integral(
        curve(&poles),
        moved,
        identity(),
        [0., 0., 1.],
        Default::default(),
        || false,
    )
    .unwrap();
    let forward = integrate(&poles, Default::default()).unwrap();
    assert_eq!(translated, forward);
    assert_eq!(integrate(&reverse, Default::default()).unwrap(), forward);
}
#[test]
fn near_parallel_direction_retains_the_smallest_nonzero_binary64_area() {
    let tiny = f64::from_bits(1);
    let line = MetricCurve::Line {
        origin: [0.; 3],
        direction: [1., tiny, 0.],
    };
    let report = certify_polynomial_side_integral(
        line,
        identity(),
        identity(),
        [1., 0., 0.],
        Default::default(),
        || false,
    )
    .unwrap();
    assert_eq!(report.enclosure(), [tiny; 2]);
    assert_eq!(report.absolute_error_bound(), 0.);
}
#[test]
fn zero_error_cap_does_not_round_nonzero_subnormal_area_to_successful_zero() {
    let line = MetricCurve::Line {
        origin: [0.; 3],
        direction: [f64::from_bits(1), 0., 0.],
    };
    let options = options_at(0);
    let error = certify_polynomial_side_integral(
        line,
        identity(),
        identity(),
        [0., 0., 0.5],
        options,
        || false,
    )
    .unwrap_err();
    assert!(matches!(*error.cause, WorkLimit { .. }));
    assert_eq!(error.diagnostic_enclosure, Some([0., f64::from_bits(1)]));
}
#[test]
fn successive_work_prefixes_tighten_enclosures_around_exact_reversal_area() {
    let poles = [[0., 0., 0.], [-1., 0., 0.], [1., 0., 0.]];
    let mut previous = [0., f64::MAX];
    let expected = ExactRatio::new(5.into(), 3.into());
    for splits in 0..=6 {
        let options = options_at(splits);
        let enclosure = integrate(&poles, options)
            .unwrap_err()
            .diagnostic_enclosure
            .unwrap();
        assert!(enclosure[0] >= previous[0] && enclosure[1] <= previous[1]);
        assert!(
            ExactRatio::from_finite(enclosure[0]) <= expected
                && expected <= ExactRatio::from_finite(enclosure[1])
        );
        previous = enclosure;
    }
}

fn options_at(splits: u32) -> PolynomialSideOptions {
    PolynomialSideOptions {
        max_subdivisions: splits,
        absolute_error_limit: 0.,
        ..Default::default()
    }
}
