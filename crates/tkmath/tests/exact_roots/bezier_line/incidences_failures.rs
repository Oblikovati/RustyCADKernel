// SPDX-License-Identifier: GPL-2.0-only
use super::incidences::request;
use super::*;

#[test]
fn exact_dyadic_parameter_reports_too_tight_then_certifies_nearest_candidate() {
    let controls = [[0., -1.], [1., 2.]];
    let mut query = request(&controls);
    query.curve.weights = Some(&[2., 1.]);
    query.max_world_error = 1e-20;
    let failure = certify_bezier_line_incidences(query, || false).unwrap_err();
    assert_eq!(failure.completion(), BezierIncidenceCompletion::Incomplete);
    assert!(matches!(
        failure.cause(),
        BezierIncidenceCause::ConstructionResolutionLimit { .. }
    ));
    let lower = failure.last_root().unwrap().bounds().0;
    assert_eq!(lower.compare_f64(0.5).unwrap(), Ordering::Equal);
    verify_repaired_world_limit(query);
}
fn verify_repaired_world_limit(mut query: BezierIncidenceQuery<'_>) {
    query.max_world_error = 3e-17;
    let repaired = certify_bezier_line_incidences(query, || false).unwrap();
    assert!(repaired.contacts()[0].achieved_world_error() <= 3e-17);
    assert_eq!(repaired.source().weights(), Some(&[2., 1.][..]));
}
#[test]
fn extrapolated_curve_parameters_preserve_original_bernstein_basis() {
    let controls = [[0., -2.], [1., -1.]];
    let mut query = request(&controls);
    query.curve.interval = [1., 3.];
    let report = certify_bezier_line_incidences(query, || false).unwrap();
    assert_eq!(report.contacts()[0].point(), [2., 0.]);
    assert_eq!(
        report.contacts()[0]
            .parameter()
            .bounds()
            .0
            .compare_f64(2.)
            .unwrap(),
        Ordering::Equal
    );
    assert_eq!(report.source().interval(), [1., 3.]);
}
#[test]
fn all_polled_cancellation_points_preserve_source_and_discard_success() {
    let controls = [[0., -1.], [1., 1.]];
    let query = request(&controls);
    let mut checkpoints = 0;
    count_checkpoints(query, &mut checkpoints);
    for stop in 1..=checkpoints {
        verify_cancelled_at(query, stop, &controls);
    }

    assert_eq!(
        certify_bezier_line_incidences(query, || false)
            .unwrap()
            .contacts()
            .len(),
        1
    );
}
fn verify_cancelled_at(query: BezierIncidenceQuery<'_>, stop: usize, controls: &[[f64; 2]]) {
    let mut polled = 0;
    let failure = certify_bezier_line_incidences(query, || {
        polled += 1;
        polled == stop
    })
    .unwrap_err();
    assert_eq!(failure.completion(), BezierIncidenceCompletion::Cancelled);
    assert_eq!(failure.source().controls(), controls);
}
#[test]
fn empty_coincident_invalid_and_exhausted_are_distinct_with_typed_sources() {
    let empty = certify_bezier_line_incidences(request(&[[0., 1.], [1., 1.]]), || false).unwrap();
    assert_eq!(empty.completion(), BezierIncidenceCompletion::Complete);
    let overlap =
        certify_bezier_line_incidences(request(&[[0., 0.], [1., 0.]]), || false).unwrap_err();
    assert_eq!(overlap.completion(), BezierIncidenceCompletion::Declined);
    let mut invalid = request(&[[0., -1.], [1., 1.]]);
    invalid.max_world_error = 0.;
    let failure = certify_bezier_line_incidences(invalid, || false).unwrap_err();
    assert_eq!(failure.completion(), BezierIncidenceCompletion::Invalid);
    assert!(std::error::Error::source(&failure).is_some());
    assert!(failure.to_string().contains("world error limit 0"));
}

fn count_checkpoints(query: BezierIncidenceQuery<'_>, checkpoints: &mut usize) {
    certify_bezier_line_incidences(query, || {
        *checkpoints += 1;
        false
    })
    .unwrap();
}

#[test]
fn failed_and_cancelled_evidence_remains_bound_after_source_overwrite() {
    let mut controls = [[0., -1.], [1., 2.]];
    let mut weights = [2., 1.];
    let mut query = request(&controls);
    query.curve.weights = Some(&weights);
    query.max_world_error = 1e-20;
    let failed = certify_bezier_line_incidences(query, || false).unwrap_err();
    let cancelled = certify_bezier_line_incidences(query, || true).unwrap_err();
    controls.fill([999., 999.]);
    weights.fill(-999.);
    for evidence in [&failed, &cancelled] {
        assert_eq!(evidence.source().controls(), &[[0., -1.], [1., 2.]]);
        assert_eq!(evidence.source().weights(), Some(&[2., 1.][..]));
        assert_eq!(evidence.source().domain_sign().version, 1);
    }
    assert_eq!(failed.completion(), BezierIncidenceCompletion::Incomplete);
    assert_eq!(cancelled.completion(), BezierIncidenceCompletion::Cancelled);
}
