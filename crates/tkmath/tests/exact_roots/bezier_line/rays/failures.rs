// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use std::error::Error;
#[test]
fn ray_coincidence_declines_after_denominator_validation() {
    let controls = [[0., 0.], [1., 0.]];
    let request = query(&controls);
    let error = classify_bezier_ray_contacts(request, || false).unwrap_err();
    assert_eq!(error, BezierRayError::SupportingLineCoincidence);
    assert!(error.to_string().contains("expected isolated contacts"));
    assert!(error.source().is_none());
}
#[test]
fn ray_denominator_failure_precedes_coincidence() {
    let mut request = query(&[[0., 0.], [1., 0.]]);
    request.weights = Some(&[0., 0.]);
    assert_eq!(
        classify_bezier_ray_contacts(request, || false),
        Err(BezierRayError::Line(BezierLineError::ZeroDenominator))
    );
    request.weights = Some(&[1., -1.]);
    assert!(matches!(
        classify_bezier_ray_contacts(request, || false),
        Err(BezierRayError::Line(BezierLineError::DenominatorRoots(_)))
    ));
}
#[test]
fn ray_empty_result_is_complete_and_distinct_from_coincidence() {
    let result = directed(query(&[[0., 1.], [1., 1.]]));
    assert!(result.signs().is_empty());
    assert!(result.isolation().roots().is_empty());
}
#[test]
fn ray_cancellation_is_observed_at_every_checkpoint() {
    for controls in [
        [[-1., 1.], [0., -2.], [1., 1.]],
        [[0., 1.], [1., 1.], [2., 1.]],
        [[0., 0.], [1., 0.], [2., 0.]],
    ] {
        check_cancellation(query(&controls));
    }
    let mut request = query(&[[0., 0.], [1., 0.]]);
    request.weights = Some(&[1., -1.]);
    check_cancellation(request);
}
fn check_cancellation(request: BezierLineQuery<'_>) {
    let mut count = 0;
    let _ = classify_bezier_ray_contacts(request, || {
        count += 1;
        false
    });
    assert!(count > 0);
    for stop in 1..=count {
        assert_cancelled_at(request, stop);
    }
}
fn assert_cancelled_at(request: BezierLineQuery<'_>, stop: usize) {
    let mut calls = 0;
    let error = classify_bezier_ray_contacts(request, || {
        calls += 1;
        calls == stop
    })
    .unwrap_err();
    assert!(error.to_string().contains("cancel"), "{stop}: {error}");
    assert!(error.source().is_some());
}

#[test]
fn ray_direction_degree_failure_preserves_its_typed_cause() {
    let controls = [[0., -1.], [0., 0.], [1., 1.]];
    let mut request = query(&controls);
    request.options.max_degree = 1;
    assert_eq!(
        classify_bezier_ray_contacts(request, || false),
        Err(BezierRayError::Direction(RootIsolationError::Degree {
            actual: 2,
            limit: 1
        }))
    );
}
#[test]
fn ray_policy_and_normal_failures_keep_the_line_stage() {
    let controls = [[0., 1.], [1., -2.], [2., 1.]];
    let mut request = query(&controls);
    request.options.version = 2;
    assert_ray_stage(
        request,
        BezierStage::Input,
        RootIsolationError::OptionsVersion(2),
    );
    request.options = options(0.0001);
    request.options.max_bisections = 0;
    assert_ray_stage(
        request,
        BezierStage::Numerator,
        RootIsolationError::SubdivisionLimit { limit: 0 },
    );
}
fn assert_ray_stage(request: BezierLineQuery<'_>, stage: BezierStage, cause: RootIsolationError) {
    assert_eq!(
        classify_bezier_ray_contacts(request, || false),
        Err(BezierRayError::Line(BezierLineError::Root { stage, cause }))
    );
}
