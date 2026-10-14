// SPDX-License-Identifier: GPL-2.0-only
use super::super::MetricBezier;
use super::tests::*;
use super::*;

fn placed(
    curve: MetricCurve<'_, 3>,
    frame: MetricEmbedding<3>,
    placement: MetricEmbedding<3>,
    vector: [f64; 3],
) -> Result<CertifiedPolynomialSideIntegral, PolynomialSideError> {
    certify_polynomial_side_integral(curve, frame, placement, vector, Default::default(), || {
        false
    })
}
fn line(direction: [f64; 3]) -> MetricCurve<'static, 3> {
    // An arbitrary finite origin must not alter the derivative integral.
    MetricCurve::Line {
        origin: [5., 6., 7.],
        direction,
    }
}
#[test]
fn invalid_policies_and_stored_degrees_are_typed_and_never_clamped() {
    let poles = [[0.; 3]; 18];
    let error = integrate(&poles, Default::default()).unwrap_err();
    assert!(matches!(
        *error.cause,
        PolynomialSideFailure::Degree {
            actual: 17,
            limit: 16
        }
    ));
    for options in invalid_policies() {
        let error = integrate(&poles[..2], options).unwrap_err();
        assert!(matches!(*error.cause, PolynomialSideFailure::Policy(_)));
        assert!(error.effective_options.is_none());
        assert_eq!(error.work, PolynomialSideWork::default());
        assert!(error.to_string().contains("expected"));
    }
}
fn invalid_policies() -> Vec<PolynomialSideOptions> {
    let mut options = vec![PolynomialSideOptions::default(); 6];
    options[0].behavior_version = 2;
    options[1].max_degree = 17;
    options[2].max_subdivisions = 65537;
    options[3].absolute_error_limit = f64::NAN;
    options[4].absolute_error_limit = f64::INFINITY;
    options[5].absolute_error_limit = -1.;
    options
}
#[test]
fn weighted_and_empty_inputs_cannot_publish_a_zero_shortcut() {
    let poles = [[0.; 3]; 2];
    let weighted = MetricCurve::Bezier(MetricBezier {
        poles: &poles,
        weights: Some(&[1., 1.]),
    });
    let error = placed(weighted, identity(), identity(), [0.; 3]).unwrap_err();
    assert!(matches!(
        *error.cause,
        PolynomialSideFailure::RationalWeights { count: 2 }
    ));
    assert!(matches!(
        *integrate(&[], Default::default()).unwrap_err().cause,
        PolynomialSideFailure::Metric(_)
    ));
}
#[test]
fn nonfinite_inputs_cannot_publish_a_zero_shortcut() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(integrate(&[[bad, 0., 0.]], Default::default()).is_err());
        let mut frame = identity();
        frame.origin[0] = bad;
        assert!(placed(line([0.; 3]), frame, identity(), [0.; 3]).is_err());
        assert!(placed(line([0.; 3]), identity(), frame, [0.; 3]).is_err());
        assert!(placed(line([0.; 3]), identity(), identity(), [bad, 0., 0.]).is_err());
    }
}
#[test]
fn oblique_affine_norm_matches_closed_form() {
    let mut frame = identity();
    frame.linear[1][0] = 2.;
    let mut placement = identity();
    placement.linear[0][1] = 3.;
    let report = placed(line([1., 0., 0.]), frame, placement, [0., 0., 1.]).unwrap();
    let [lower, upper] = report.enclosure().map(ExactRatio::from_finite);
    let squared = ExactRatio::from_finite(53.);
    assert!(lower.multiply(&lower) <= squared && upper.multiply(&upper) >= squared);
}
#[test]
fn affine_composition_precedes_rounding_of_extreme_intermediates() {
    let mut frame = identity();
    frame.scale = 2.;
    let mut placement = identity();
    placement.scale = 0.5;
    let report = placed(line([f64::MAX, 0., 0.]), frame, placement, [0., 0., 1.]).unwrap();
    assert_eq!(report.enclosure(), [f64::MAX; 2]);
}
#[test]
fn subnormal_area_retains_nonzero_upper_and_error_bound() {
    let tiny = f64::from_bits(1);
    let report = placed(line([tiny, 0., 0.]), identity(), identity(), [0., 0., 0.5]).unwrap();
    assert_eq!(report.enclosure(), [0., tiny]);
    assert_eq!(report.absolute_error_bound(), tiny);
}
#[test]
fn unrepresentable_area_bounds_are_typed_incomplete() {
    let error = placed(
        line([f64::MAX, 0., 0.]),
        identity(),
        identity(),
        [0., 0., 2.],
    )
    .unwrap_err();
    assert!(matches!(
        *error.cause,
        PolynomialSideFailure::Unrepresentable { .. }
    ));
    assert!(error.diagnostic_enclosure.is_none());
}
#[test]
fn exhausted_zero_budget_retains_bounds_and_actual_work_without_success() {
    let poles = [[0., 0., 0.], [-1., 0., 0.], [1., 0., 0.]];
    for splits in [0, 1, 4] {
        let options = PolynomialSideOptions {
            max_subdivisions: splits,
            absolute_error_limit: 0.,
            ..Default::default()
        };
        let error = integrate(&poles, options).unwrap_err();
        assert!(
            matches!(*error.cause, PolynomialSideFailure::WorkLimit { requested, .. } if requested == splits)
        );
        assert_eq!(error.work.subdivisions, splits);
        assert_eq!(error.work.interval_bounds, 1 + 2 * splits);
        assert!(error.diagnostic_enclosure.is_some());
        assert_eq!(error.effective_options, Some(options));
    }
}
#[test]
fn polynomial_degree_ladder_and_degenerate_curves_use_one_pipeline() {
    for degree in 0..=16 {
        let poles: Vec<[f64; 3]> = (0..=degree).map(|index| [index as f64, 0., 0.]).collect();
        let options = PolynomialSideOptions {
            max_degree: degree,
            absolute_error_limit: 0.,
            ..Default::default()
        };
        let report = integrate(&poles, options).unwrap();
        assert_eq!(report.enclosure(), [f64::from(degree); 2]);
        assert_eq!(report.work().subdivisions, 0);
    }
}
